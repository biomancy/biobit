use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use ::higher_kinded_types::prelude::*;
use eyre::{eyre, Result};
use rayon::ThreadPool;
use thread_local::ThreadLocal;

use biobit_collections_rs::genomic_index::GenomicIndex;
use biobit_collections_rs::interval_tree::ITree;
use biobit_core_rs::{
    loc::{AsLocus, Contig},
    num::{Float, PrimInt},
};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;

use super::result::Stats;
use super::worker::Worker;

#[derive(Debug)]
pub struct Engine<Ctg, Idx, Cnts>
where
    Ctg: Contig + Send,
    Idx: PrimInt + Send,
    Cnts: Float + Send,
{
    workers: ThreadLocal<RefCell<Worker<Ctg, Idx, Cnts>>>,
}

impl<Ctg, Idx, Cnts> Default for Engine<Ctg, Idx, Cnts>
where
    Ctg: Contig + Send,
    Idx: PrimInt + Send,
    Cnts: Float + Send,
{
    fn default() -> Self {
        Self {
            workers: ThreadLocal::new(),
        }
    }
}

impl<Ctg, Idx, Cnts> Engine<Ctg, Idx, Cnts>
where
    Ctg: Contig + Send + Sync,
    Idx: PrimInt + Send,
    Cnts: Float + Send,
{
    pub fn run<Src, IT, Lcs>(
        &mut self,
        pool: &mut ThreadPool,
        objects: usize,
        sources: &[Src],
        index: &GenomicIndex<Ctg, IT>,
        partitions: &[Lcs],
    ) -> Result<Vec<(Vec<Cnts>, Vec<Stats<Ctg, Idx, Cnts>>)>>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
        IT: ITree<Idx = Idx, Value = usize> + Sync,
        Lcs: AsLocus<Contig = Ctg, Idx = Idx> + Sync,
    {
        // Soft-reset all workers
        for w in self.workers.iter_mut() {
            w.borrow_mut().reset()
        }

        // Run the counting
        let worker_sources: ThreadLocal<RefCell<Vec<Src>>> = ThreadLocal::new();
        let error_occured = AtomicBool::new(false);
        let errors = Mutex::new(Vec::new());

        let inds = (0..sources.len()).collect::<Vec<_>>();
        pool.scope(|s| {
            for ind in &inds {
                for partition in partitions {
                    // Terminate the loop if an error has occured in any of the threads
                    if error_occured.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }

                    s.spawn(|_| {
                        if error_occured.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }

                        let mut sources = worker_sources
                            .get_or(|| {
                                RefCell::new(sources.iter().map(|x| dyn_clone::clone(x)).collect())
                            })
                            .borrow_mut();

                        let mut worker = self.workers.get_or_default().borrow_mut();

                        let result =
                            worker.calculate(*ind, objects, index, &mut sources[*ind], partition);
                        if let Err(err) = result {
                            error_occured.store(true, std::sync::atomic::Ordering::Relaxed);
                            errors
                                .lock()
                                .expect("TODO: Failed to hold the mutex")
                                .push(err);
                        }
                    });
                }
            }
        });

        let error_occured = error_occured.into_inner();
        if error_occured {
            let errors = errors.into_inner()?;
            return Err(eyre!("CountIt failed. Errors: {:?}", errors));
        }

        let collapsed = Worker::collapse(
            sources.len(),
            objects,
            partitions.len(),
            self.workers.iter_mut().map(|x| x.get_mut()),
        );

        Ok(collapsed)
    }
}
