use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use biobit_core_rs::loc::IntervalOp;
use biobit_core_rs::num::PrimUInt;
use biobit_core_rs::source::Source;
use biobit_io_rs::fasta::IndexedSources;
use eyre::{Result, eyre};
use rayon::ThreadPool;
use thread_local::ThreadLocal;

use crate::pileup::SparsePileup;
use crate::result::SelectedPileup;
use crate::selection::Selector;
use crate::task::Task;
use crate::worker::{DynReadSource, SourceArgs, SourceItem, Worker};

type SourceMap<SeqId> = RefCell<HashMap<usize, Vec<DynReadSource<SeqId>>>>;
type ResultMap<SeqId, Idx, Cnts> = RefCell<HashMap<usize, Vec<SparsePileup<SeqId, Idx, Cnts>>>>;

pub struct Reat<SeqId, Idx, Cnts, SmplTag, Src>
where
    Idx: PrimUInt + Send,
    Cnts: PrimUInt + Send,
    SeqId: Send,
    SmplTag: Ord,
{
    pool: ThreadPool,
    reference: IndexedSources,
    selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
    min_phred: u8,
    samples: BTreeMap<SmplTag, Vec<Src>>,
}

impl<SeqId, Idx, Cnts, SmplTag, Src> Reat<SeqId, Idx, Cnts, SmplTag, Src>
where
    SeqId: AsRef<str> + Clone + Ord + PartialEq + Send + Sync + 'static,
    Idx: PrimUInt + Send + Sync + 'static,
    Cnts: PrimUInt + Send + Sync + 'static,
    SmplTag: Clone + Ord,
    Src: Source<Args = SourceArgs<SeqId>, Item = SourceItem> + 'static,
{
    pub fn new(
        pool: ThreadPool,
        reference: IndexedSources,
        min_phred: u8,
        selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
    ) -> Self {
        Self {
            pool,
            reference,
            selector,
            min_phred,
            samples: BTreeMap::new(),
        }
    }

    pub fn register<Sources>(&mut self, tag: SmplTag, sources: Sources) -> &mut Self
    where
        Sources: IntoIterator<Item = Src>,
    {
        self.samples.entry(tag).or_default().extend(sources);
        self
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }

    pub fn run<Tasks>(
        &mut self,
        tasks: Tasks,
    ) -> Result<Vec<SelectedPileup<SeqId, Idx, Cnts, SmplTag>>>
    where
        Tasks: IntoIterator<Item = Task<SeqId, Idx>>,
    {
        let tasks = tasks.into_iter().collect::<Vec<_>>();
        let size_hint = max_task_envelope(&tasks)?;

        let tags = self.samples.keys().cloned().collect::<Vec<_>>();
        let sources = self.samples.values().collect::<Vec<_>>();

        let workers = ThreadLocal::<RefCell<Worker<SeqId, Idx, Cnts>>>::new();
        let source_cache = ThreadLocal::<SourceMap<SeqId>>::new();
        let result_cache = ThreadLocal::<ResultMap<SeqId, Idx, Cnts>>::new();
        let errors = ThreadLocal::<RefCell<Vec<eyre::Report>>>::new();
        let error_occurred = AtomicBool::new(false);

        let sample_indices = (0..tags.len()).collect::<Vec<_>>();
        let task_indices = (0..tasks.len()).collect::<Vec<_>>();

        let reference = self.reference.clone();
        let selector = Arc::clone(&self.selector);
        let min_phred = self.min_phred;

        self.pool.scope(|scope| {
            for sample_index in &sample_indices {
                for task_index in &task_indices {
                    if error_occurred.load(Ordering::Relaxed) {
                        return;
                    }

                    scope.spawn(|_| {
                        if error_occurred.load(Ordering::Relaxed) {
                            return;
                        }

                        let result = (|| -> Result<()> {
                            let worker = workers.get_or_try(|| {
                                Ok::<_, eyre::Report>(RefCell::new(Worker::new(
                                    reference.open()?,
                                    Arc::clone(&selector),
                                    min_phred,
                                    size_hint,
                                )))
                            })?;
                            let mut worker = worker.borrow_mut();

                            let mut source_cache = source_cache.get_or_default().borrow_mut();
                            let sample_sources = source_cache
                                .entry(*sample_index)
                                .or_insert_with(|| clone_sources(sources[*sample_index]));

                            let pileups = worker.process(&tasks[*task_index], sample_sources)?;
                            result_cache
                                .get_or_default()
                                .borrow_mut()
                                .entry(*sample_index)
                                .or_default()
                                .extend(pileups);
                            Ok(())
                        })();

                        if let Err(err) = result {
                            error_occurred.store(true, Ordering::Relaxed);
                            errors.get_or_default().borrow_mut().push(err);
                        }
                    });
                }
            }
        });

        if error_occurred.into_inner() {
            let mut collapsed_errors = Vec::new();
            for errors in errors {
                collapsed_errors.extend(errors.into_inner());
            }
            return Err(eyre!("REAT failed. Errors: {:?}", collapsed_errors));
        }

        let mut collapsed = (0..tags.len()).map(|_| Vec::new()).collect::<Vec<_>>();
        for results in result_cache {
            for (sample_index, pileups) in results.into_inner() {
                collapsed[sample_index].extend(pileups);
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

fn max_task_envelope<SeqId, Idx: PrimUInt>(tasks: &[Task<SeqId, Idx>]) -> Result<usize> {
    let mut hint = 0;
    for task in tasks {
        let len = task
            .envelope()
            .len()
            .to_usize()
            .ok_or_else(|| eyre!("task envelope length does not fit into usize"))?;
        hint = hint.max(len);
    }
    Ok(hint)
}
