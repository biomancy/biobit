use crate::rigid::resolution::Resolution;
use crate::rigid::{EngineBuilder, Partition, Worker};
use crate::Counts;
use biobit_core_rs::loc::Contig;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;
use higher_kinded_types::extra_arities::For;
use rayon::ThreadPool;
use std::borrow::BorrowMut;
use std::cell::{Ref, RefCell};
use thread_local::ThreadLocal;

use ahash::HashMap;
use derive_more::Constructor;
use eyre::eyre;
use itertools::izip;
use std::io;
use std::iter::zip;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

#[derive(Constructor)]
pub struct Engine<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> {
    thread_pool: Option<ThreadPool>,
    elements: Vec<Elt>,
    workers: ThreadLocal<RefCell<Worker<Ctg, Idx, Cnts, Elt>>>,
    partitions: Vec<Partition<Ctg, Idx, Elt>>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> Engine<Ctg, Idx, Cnts, Elt>
where
    Elt: Send + Sync + Clone,
{
    pub fn builder() -> EngineBuilder<Ctg, Idx, Elt> {
        EngineBuilder::default()
    }

    pub fn run<SrcTag, Src>(
        &mut self,
        sources: impl Iterator<Item = (SrcTag, Src)>,
        resolution: Box<dyn Resolution<Idx, Cnts, Elt>>,
    ) -> eyre::Result<Vec<Counts<Ctg, Idx, Cnts, Elt, SrcTag>>>
    where
        SrcTag: Send,
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        let (tags, sources): (Vec<_>, Vec<_>) = sources.unzip();
        match self.thread_pool.take() {
            Some(pool) => {
                let result = pool.install(|| self._run(tags, sources, resolution));
                self.thread_pool = Some(pool);
                result
            }
            None => self._run(tags, sources, resolution),
        }
    }

    pub fn _run<SrcTag, Src>(
        &mut self,
        tags: Vec<SrcTag>,
        sources: Vec<Src>,
        resolution: Box<dyn Resolution<Idx, Cnts, Elt>>,
    ) -> eyre::Result<Vec<Counts<Ctg, Idx, Cnts, Elt, SrcTag>>>
    where
        SrcTag: Send,
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        // Soft-reset all workers
        for w in self.workers.iter_mut() {
            w.borrow_mut()
                .get_mut()
                .reset(dyn_clone::clone_box(&*resolution));
        }

        // Run the counting
        let worker_sources: ThreadLocal<RefCell<HashMap<usize, Src>>> = ThreadLocal::new();
        let has_failed = AtomicBool::new(false);

        let _srcinds = (0..sources.len()).collect::<Vec<_>>();
        let _prtinds = (0..self.partitions.len()).collect::<Vec<_>>();
        rayon::scope(|s| {
            for srcind in &_srcinds {
                for prtind in &_prtinds {
                    // Terminate the loop if an error has occured in any of the threads
                    if has_failed.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }

                    s.spawn(|_| {
                        if has_failed.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }

                        // Get the local copy of the target source (or copy it if it does not exist)
                        let mut local_sources = worker_sources.get_or_default().borrow_mut();
                        let source = local_sources
                            .entry(*srcind)
                            .or_insert_with(|| dyn_clone::clone(&sources[*srcind]));

                        // Get the state of the worker (or create a new one if it does not exist)
                        let mut worker = self
                            .workers
                            .get_or(|| {
                                RefCell::new(Worker::new(dyn_clone::clone_box(&*resolution)))
                            })
                            .borrow_mut();

                        let result =
                            worker.process(*srcind, source, *prtind, &self.partitions[*prtind]);

                        if let Err(err) = result {
                            has_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                            log::error!("CountIt failed: {:?}", err);
                        }
                    });
                }
            }
        });

        let has_failed = has_failed.into_inner();
        if has_failed {
            return Err(eyre!("CountIt internal error. See log for details."));
        }

        let collapsed = Worker::aggregate(
            sources.len(),
            self.elements.len(),
            &self.partitions,
            self.workers.iter_mut().map(|x| x.get_mut()),
        );
        let result = collapsed
            .into_iter()
            .zip(tags)
            .map(|((cnts, stats), tag)| Counts::new(tag, self.elements.clone(), cnts, stats))
            .collect();

        Ok(result)
    }
}
