use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
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

struct ThreadLocalCache<SeqId, Idx, Cnts>
where
    SeqId: Send,
    Idx: PrimUInt + Send,
    Cnts: PrimUInt + Send,
{
    workers: ThreadLocal<RefCell<Worker<SeqId, Idx, Cnts>>>,
    sources: ThreadLocal<SourceMap<SeqId>>,
    results: ThreadLocal<ResultMap<SeqId, Idx, Cnts>>,
    errors: ThreadLocal<RefCell<Vec<eyre::Report>>>,
    failed: AtomicBool,
}

impl<SeqId, Idx, Cnts> ThreadLocalCache<SeqId, Idx, Cnts>
where
    SeqId: Send,
    Idx: PrimUInt + Send,
    Cnts: PrimUInt + Send,
{
    fn new() -> Self {
        Self {
            workers: ThreadLocal::new(),
            sources: ThreadLocal::new(),
            results: ThreadLocal::new(),
            errors: ThreadLocal::new(),
            failed: AtomicBool::new(false),
        }
    }

    fn failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    fn worker(
        &self,
        new: impl FnOnce() -> Result<Worker<SeqId, Idx, Cnts>>,
    ) -> Result<RefMut<'_, Worker<SeqId, Idx, Cnts>>> {
        let worker = self.workers.get_or_try(|| new().map(RefCell::new))?;
        Ok(worker.borrow_mut())
    }

    fn sources<Src>(
        &self,
        sample_index: usize,
        sources: &[Src],
    ) -> Result<RefMut<'_, Vec<DynReadSource<SeqId>>>>
    where
        Src: Source<Args = SourceArgs<SeqId>, Item = SourceItem> + 'static,
    {
        let cache = self
            .sources
            .get_or_try(|| Ok::<_, eyre::Report>(RefCell::new(HashMap::new())))?;
        Ok(RefMut::map(cache.borrow_mut(), |cache| {
            cache.entry(sample_index).or_insert_with(|| {
                sources
                    .iter()
                    .map(|source| {
                        Box::new(dyn_clone::clone(source).to_dynsrc()) as DynReadSource<SeqId>
                    })
                    .collect()
            })
        }))
    }

    fn add_result(
        &self,
        sample_index: usize,
        pileups: Vec<SparsePileup<SeqId, Idx, Cnts>>,
    ) -> Result<()> {
        let results = self
            .results
            .get_or_try(|| Ok::<_, eyre::Report>(RefCell::new(HashMap::new())))?;
        results
            .borrow_mut()
            .entry(sample_index)
            .or_default()
            .extend(pileups);
        Ok(())
    }

    fn record_error(&self, err: eyre::Report) {
        self.failed.store(true, Ordering::Relaxed);
        let errors = self
            .errors
            .get_or_try(|| Ok::<_, Infallible>(RefCell::new(Vec::new())))
            .unwrap_or_else(|never| match never {});
        errors.borrow_mut().push(err);
    }

    fn into_errors(self) -> Vec<eyre::Report> {
        let mut collapsed = Vec::new();
        for errors in self.errors {
            collapsed.extend(errors.into_inner());
        }
        collapsed
    }

    fn into_results(self, samples: usize) -> Vec<Vec<SparsePileup<SeqId, Idx, Cnts>>> {
        let mut collapsed = (0..samples).map(|_| Vec::new()).collect::<Vec<_>>();
        for results in self.results {
            for (sample_index, pileups) in results.into_inner() {
                collapsed[sample_index].extend(pileups);
            }
        }
        collapsed
    }
}

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

        // Order is preserved by BTreeMap, so tags and sources are aligned by index.
        let tags = self.samples.keys().cloned().collect::<Vec<_>>();
        let sources = self.samples.values().collect::<Vec<_>>();

        let cache = ThreadLocalCache::<SeqId, Idx, Cnts>::new();

        let sample_indices = (0..tags.len()).collect::<Vec<_>>();
        let task_indices = (0..tasks.len()).collect::<Vec<_>>();

        self.pool.scope(|scope| {
            for smplidx in &sample_indices {
                for taskidx in &task_indices {
                    if cache.failed() {
                        return;
                    }

                    scope.spawn(|_| {
                        if cache.failed() {
                            return;
                        }

                        let result = (|| -> Result<()> {
                            let mut worker = cache.worker(|| {
                                Ok(Worker::new(
                                    self.reference.open()?,
                                    Arc::clone(&self.selector),
                                    self.min_phred,
                                    size_hint,
                                ))
                            })?;
                            let mut smplsrc = cache.sources(*smplidx, sources[*smplidx])?;

                            let result =
                                worker.process(&tasks[*taskidx], smplsrc.as_mut_slice())?;
                            cache.add_result(*smplidx, result)?;
                            Ok(())
                        })();

                        if let Err(err) = result {
                            cache.record_error(err);
                        }
                    });
                }
            }
        });

        if cache.failed() {
            return Err(eyre!("REAT failed. Errors: {:?}", cache.into_errors()));
        }

        let collapsed = cache.into_results(tags.len());
        tags.into_iter()
            .zip(collapsed)
            .map(|(tag, pileups)| SelectedPileup::new(tag, pileups))
            .collect()
    }
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
