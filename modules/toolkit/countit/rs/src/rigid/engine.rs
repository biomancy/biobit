use crate::Counts;
use crate::rigid::resolution::Resolution;
use crate::rigid::{EngineBuilder, Partition, Worker};
use biobit_core_rs::loc::Contig;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;
use rayon::ThreadPool;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use thread_local::ThreadLocal;

use ahash::HashMap;
use derive_more::Constructor;
use higher_kinded_types::ForLt;
use std::io;
use std::sync::OnceLock;

#[derive(Constructor)]
// derive_more::Constructor generates explicit field names.
#[allow(clippy::redundant_field_names)]
pub struct Engine<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> {
    thread_pool: Option<ThreadPool>,
    elements: Vec<Elt>,
    workers: ThreadLocal<RefCell<Worker<Ctg, Idx, Cnts, Elt>>>,
    partitions: Vec<Partition<Ctg, Idx>>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> Engine<Ctg, Idx, Cnts, Elt>
where
    Elt: Send + Sync + Clone,
{
    pub fn builder() -> EngineBuilder<Ctg, Idx, Elt> {
        EngineBuilder::default()
    }

    #[allow(clippy::type_complexity)]
    pub fn run<SrcTag, Src>(
        &mut self,
        sources: impl Iterator<Item = (SrcTag, Src)>,
        resolution: Box<dyn Resolution<Idx, Cnts, Elt>>,
    ) -> eyre::Result<Vec<Counts<'_, Ctg, Idx, Cnts, Elt, SrcTag>>>
    where
        SrcTag: Send,
        Src: Source<
                Args = ForLt!(<'args> = (&'args Ctg, Idx, Idx)),
                Item = ForLt!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
            >,
    {
        let (tags, sources): (Vec<_>, Vec<_>) = sources.unzip();
        match self.thread_pool.take() {
            Some(pool) => {
                pool.install(|| self._run(sources, resolution))?;
                self.thread_pool = Some(pool);
            }
            None => self._run(sources, resolution)?,
        }

        let collapsed = Worker::aggregate(
            tags.len(),
            self.elements.len(),
            &self.partitions,
            self.workers.iter_mut().map(|x| x.get_mut()),
        );
        let result = collapsed
            .into_iter()
            .zip(tags)
            .map(|((cnts, stats), tag)| Counts {
                source: tag,
                elements: &self.elements,
                counts: cnts,
                partitions: stats,
            })
            .collect();

        Ok(result)
    }

    pub fn _run<Src>(
        &mut self,
        sources: Vec<Src>,
        resolution: Box<dyn Resolution<Idx, Cnts, Elt>>,
    ) -> eyre::Result<()>
    where
        Src: Source<
                Args = ForLt!(<'args> = (&'args Ctg, Idx, Idx)),
                Item = ForLt!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
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
        // Retain the first worker error; its presence also stops further scheduling.
        let failure = OnceLock::new();

        let _srcinds = (0..sources.len()).collect::<Vec<_>>();
        let _prtinds = (0..self.partitions.len()).collect::<Vec<_>>();
        rayon::scope(|s| {
            for srcind in &_srcinds {
                for prtind in &_prtinds {
                    // Stop scheduling if any worker has failed.
                    if failure.get().is_some() {
                        return;
                    }

                    s.spawn(|_| {
                        if failure.get().is_some() {
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

                        let result = worker.process(
                            &self.elements,
                            *srcind,
                            source,
                            *prtind,
                            &self.partitions[*prtind],
                        );

                        if let Err(err) = result {
                            log::error!("CountIt failed: {:?}", err);
                            let _ = failure.set(err.wrap_err(format!(
                                "CountIt failed for source {}, partition {} ({:?}, {:?})",
                                *srcind,
                                *prtind,
                                self.partitions[*prtind].contig(),
                                self.partitions[*prtind].interval(),
                            )));
                        }
                    });
                }
            }
        });

        match failure.into_inner() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
