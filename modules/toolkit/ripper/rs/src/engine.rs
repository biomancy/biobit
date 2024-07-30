use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use ::higher_kinded_types::prelude::*;
use ahash::HashMap;
use derive_getters::Dissolve;
use derive_more::Constructor;
use eyre::{eyre, Result};
use rayon::ThreadPool;
use thread_local::ThreadLocal;

use biobit_core_rs::{
    loc::Contig,
    num::{Float, PrimInt},
};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;

use super::config::Config;
use super::result::Ripped;
use super::worker::Worker;

#[derive(Constructor, Dissolve)]
pub struct Comparison<Idx: PrimInt, Cnts: Float, Tag, Src> {
    pub tag: Tag,
    pub signal: Vec<Src>,
    pub control: Vec<Src>,
    pub config: Config<Idx, Cnts>,
}

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
    Idx: PrimInt + Send + Sync,
    Cnts: Float + Send + Sync,
{
    pub fn reset(&mut self) {
        for w in self.workers.iter_mut() {
            w.borrow_mut().reset()
        }
    }

    pub fn run<Tag, Src>(
        &mut self,
        pool: &mut ThreadPool,
        queries: Vec<(Ctg, Idx, Idx)>,
        comparisons: Vec<Comparison<Idx, Cnts, Tag, Src>>,
    ) -> Result<Vec<Ripped<Ctg, Idx, Cnts, Tag>>>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        // Soft-reset all workers
        for w in self.workers.iter_mut() {
            w.borrow_mut().reset()
        }

        // Dissolve the comparisons
        let (tags, signals, controls, configs) = comparisons.into_iter().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut tags, mut signals, mut controls, mut configs), comparison| {
                tags.push(comparison.tag);
                signals.push(comparison.signal);
                controls.push(comparison.control);
                configs.push(comparison.config);
                (tags, signals, controls, configs)
            },
        );

        // Run the counting
        let sources: ThreadLocal<RefCell<HashMap<usize, (Vec<Src>, Vec<Src>)>>> =
            ThreadLocal::new();

        let error_occured = AtomicBool::new(false);
        let errors = Mutex::new(Vec::new());

        let cmpinds = (0..tags.len()).collect::<Vec<_>>();
        let queryinds = (0..queries.len()).collect::<Vec<_>>();
        pool.scope(|s| {
            for cmpind in &cmpinds {
                for queryind in &queryinds {
                    // Terminate the loop if an error has occured in any of the threads
                    if error_occured.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }

                    s.spawn(|_| {
                        if error_occured.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }

                        let mut sources = sources.get_or_default().borrow_mut();
                        let (signal, control) = sources.entry(*cmpind).or_insert_with(|| {
                            (
                                signals[*cmpind]
                                    .iter()
                                    .map(|x| dyn_clone::clone(x))
                                    .collect(),
                                controls[*cmpind]
                                    .iter()
                                    .map(|x| dyn_clone::clone(x))
                                    .collect(),
                            )
                        });

                        let result = self.workers.get_or_default().borrow_mut().calculater(
                            *cmpind,
                            *queryind,
                            &queries[*queryind],
                            signal,
                            control,
                            &configs[*cmpind],
                        );
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
            return Err(eyre!("Ripper failed. Errors: {:?}", errors));
        }

        let collapsed =
            Worker::collapse(tags, queries, self.workers.iter_mut().map(|x| x.get_mut()));

        Ok(collapsed)
    }
}
