use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use biobit_core_rs::loc::IntervalOp;
use biobit_core_rs::num::PrimUInt;
use biobit_core_rs::source::Source;
use biobit_io_rs::fasta::IndexedReaderMutOp;
use eyre::{Result, eyre};
use rayon::ThreadPool;
use thread_local::ThreadLocal;

use crate::result::SelectedPileup;
use crate::selection::Selector;
use crate::worker::{DynReadSource, SourceArgs, SourceItem, Worker};
use crate::workload::Workload;

pub type ReferenceFactory =
    dyn Fn() -> Result<Box<dyn IndexedReaderMutOp + Send + Sync>> + Send + Sync;

type WorkerMap<SeqId, Idx, Cnts> = RefCell<HashMap<usize, Worker<SeqId, Idx, Cnts>>>;

pub struct Analysis<SeqId, Idx: PrimUInt, Cnts: PrimUInt, Tag, Src> {
    pub tag: Tag,
    pub sources: Vec<Src>,
    pub workload: Workload<SeqId, Idx>,
    pub selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
    pub min_phred: u8,
}

impl<SeqId, Idx, Cnts, Tag, Src> Analysis<SeqId, Idx, Cnts, Tag, Src>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(
        tag: Tag,
        sources: Vec<Src>,
        workload: Workload<SeqId, Idx>,
        selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
        min_phred: u8,
    ) -> Self {
        Self {
            tag,
            sources,
            workload,
            selector,
            min_phred,
        }
    }
}

pub struct Engine<SeqId: Send, Idx: PrimUInt + Send, Cnts: PrimUInt + Send> {
    workers: ThreadLocal<WorkerMap<SeqId, Idx, Cnts>>,
}

impl<SeqId, Idx, Cnts> Default for Engine<SeqId, Idx, Cnts>
where
    SeqId: Send,
    Idx: PrimUInt + Send,
    Cnts: PrimUInt + Send,
{
    fn default() -> Self {
        Self {
            workers: ThreadLocal::new(),
        }
    }
}

impl<SeqId, Idx, Cnts> Engine<SeqId, Idx, Cnts>
where
    SeqId: AsRef<str> + Clone + Default + Ord + PartialEq + Send + Sync + 'static,
    Idx: PrimUInt + Send + Sync + 'static,
    Cnts: PrimUInt + Send + Sync + 'static,
{
    pub fn reset(&mut self) {
        self.workers.clear();
    }

    pub fn run<Tag, Src>(
        &mut self,
        pool: &mut ThreadPool,
        analyses: Vec<Analysis<SeqId, Idx, Cnts, Tag, Src>>,
        reference_factory: Arc<ReferenceFactory>,
    ) -> Result<Vec<SelectedPileup<SeqId, Idx, Cnts, Tag>>>
    where
        Src: Source<Args = SourceArgs<SeqId>, Item = SourceItem> + 'static,
    {
        self.reset();

        let mut tags = Vec::with_capacity(analyses.len());
        let mut sources = Vec::with_capacity(analyses.len());
        let mut workloads = Vec::with_capacity(analyses.len());
        let mut selectors = Vec::with_capacity(analyses.len());
        let mut min_phreds = Vec::with_capacity(analyses.len());
        let mut size_hints = Vec::with_capacity(analyses.len());

        for analysis in analyses {
            size_hints.push(size_hint(&analysis.workload)?);
            tags.push(analysis.tag);
            sources.push(analysis.sources);
            workloads.push(analysis.workload);
            selectors.push(analysis.selector);
            min_phreds.push(analysis.min_phred);
        }

        let errors = Mutex::new(Vec::new());
        let error_occurred = AtomicBool::new(false);

        let analysis_indices = (0..tags.len()).collect::<Vec<_>>();
        let max_tasks = workloads
            .iter()
            .map(|workload| workload.tasks.len())
            .max()
            .unwrap_or(0);
        let task_indices = (0..max_tasks).collect::<Vec<_>>();

        pool.scope(|scope| {
            for analysis_index in &analysis_indices {
                for task_index in &task_indices {
                    if *task_index >= workloads[*analysis_index].tasks.len() {
                        continue;
                    }
                    if error_occurred.load(Ordering::Relaxed) {
                        return;
                    }

                    scope.spawn(|_| {
                        if error_occurred.load(Ordering::Relaxed) {
                            return;
                        }

                        let result = (|| -> Result<()> {
                            let task = workloads[*analysis_index].tasks[*task_index].clone();

                            let mut workers = self.workers.get_or_default().borrow_mut();
                            let worker = match workers.entry(*analysis_index) {
                                std::collections::hash_map::Entry::Occupied(entry) => {
                                    entry.into_mut()
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    let worker = Worker::new(
                                        clone_sources(&sources[*analysis_index]),
                                        reference_factory()?,
                                        Arc::clone(&selectors[*analysis_index]),
                                        min_phreds[*analysis_index],
                                        size_hints[*analysis_index],
                                    );
                                    entry.insert(worker)
                                }
                            };

                            worker.process(task)
                        })();

                        if let Err(err) = result {
                            error_occurred.store(true, Ordering::Relaxed);
                            errors
                                .lock()
                                .expect("failed to hold REAT error mutex")
                                .push(err);
                        }
                    });
                }
            }
        });

        if error_occurred.into_inner() {
            self.reset();
            let errors = errors.into_inner()?;
            return Err(eyre!("REAT failed. Errors: {:?}", errors));
        }

        let mut collapsed = (0..tags.len()).map(|_| Vec::new()).collect::<Vec<_>>();
        let workers = std::mem::take(&mut self.workers);
        for workers in workers {
            for (analysis_index, worker) in workers.into_inner() {
                for (_, results) in worker.dissolve() {
                    collapsed[analysis_index]
                        .extend(results.into_iter().map(|(_task, pileup)| pileup));
                }
            }
        }

        tags.into_iter()
            .zip(collapsed)
            .map(|(tag, pileups)| SelectedPileup::new(tag, pileups))
            .collect()
    }
}

fn clone_sources<SeqId, Src>(sources: &[Src]) -> Vec<DynReadSource<SeqId>>
where
    Src: Source<Args = SourceArgs<SeqId>, Item = SourceItem> + 'static,
{
    sources
        .iter()
        .map(|source| Box::new(dyn_clone::clone(source).to_dynsrc()) as DynReadSource<SeqId>)
        .collect()
}

fn size_hint<SeqId, Idx: PrimUInt>(workload: &Workload<SeqId, Idx>) -> Result<usize> {
    let mut hint = 0;
    for task in &workload.tasks {
        let len = task
            .envelope
            .len()
            .to_usize()
            .ok_or_else(|| eyre!("task envelope length does not fit into usize"))?;
        hint = hint.max(len);
    }
    Ok(hint)
}
