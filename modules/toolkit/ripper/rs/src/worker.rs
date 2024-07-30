use ::higher_kinded_types::prelude::*;
use ahash::HashMap;
use derive_getters::Dissolve;
use derive_more::Constructor;
pub use eyre::Result;
use itertools::izip;

use biobit_collections_rs::rle_vec;
use biobit_collections_rs::rle_vec::{Merge2Fn, RleVec};
use biobit_core_rs::{
    loc::Contig,
    num::{Float, PrimInt},
    source::AnyMap,
};
use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{AsSegment, PerOrientation, Segment};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;

use super::config::Config;
use super::pcalling;
use super::result::{Region, Ripped};

#[derive(Debug, Default, Dissolve, Constructor)]
pub struct RleIdentical<Cnts: Float> {
    pub sensitivity: Cnts,
}

impl<Cnts: Float> rle_vec::Identical<Cnts> for RleIdentical<Cnts> {
    fn identical(&self, first: &Cnts, second: &Cnts) -> bool {
        first.abs_sub(*second) < self.sensitivity
    }
}

#[derive(Debug, Default, Dissolve)]
pub struct Worker<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // (Comparison ID, Query ID) -> pcalling results
    comparisons: HashMap<(usize, usize), Region<Ctg, Idx, Cnts>>,
    // Internal caches
    rle_cache: Vec<rle_vec::RleVec<Cnts, u32, RleIdentical<Cnts>>>,
    cnts_cache: Vec<PerOrientation<Vec<Cnts>>>,
    // Cache for sources
    sources_cache: AnyMap,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Worker<Ctg, Idx, Cnts> {
    fn pileup<Src>(
        &mut self,
        query: (&Ctg, Idx, Idx),
        sources: &mut [Src],
    ) -> Result<PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        let mut result = self.cnts_cache.pop().unwrap_or_default();
        result.apply(|x| {
            x.clear();
            x.resize((query.2 - query.1).to_usize().unwrap(), Cnts::zero());
        });

        // let (mut inside_annotation, mut outside_annotation) = (Cnts::zero(), Cnts::zero());
        let (start, end) = (query.1.to_usize().unwrap(), query.2.to_usize().unwrap());

        for src in sources {
            src.populate_caches(&mut self.sources_cache);

            {
                let mut iter = src.fetch(query)?;
                while let Some(blocks) = iter.next() {
                    for (segments, orientation, n) in blocks?.iter() {
                        let weight = Cnts::one() / Cnts::from(n).unwrap();
                        let saveto = result.get_mut(orientation);

                        for s in segments {
                            let segment_start = s.start().to_usize().unwrap().max(start) - start;
                            let segment_end = s.end().to_usize().unwrap().min(end) - start;
                            for i in segment_start..segment_end {
                                saveto[i] = saveto[i] + weight;
                            }
                            // let consumed_length = Cnts::from(segment_end - segment_start).unwrap();
                            // let real_length = Cnts::from(s.len()).unwrap();
                            // inside_annotation = inside_annotation + consumed_length * weight;
                            // outside_annotation =
                            //     outside_annotation + (real_length - consumed_length) * weight;
                        }
                    }
                }
            }
            src.release_caches(&mut self.sources_cache);
        }
        todo!()
    }

    pub fn reset(&mut self) {
        self.comparisons.clear();
    }

    pub fn calculater<Src>(
        &mut self,
        cmpind: usize,
        queryind: usize,
        query: &(Ctg, Idx, Idx),
        signal: &mut [Src],
        control: &mut [Src],
        config: &Config<Idx, Cnts>,
    ) -> Result<()>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        // TODO: recycle RLE and other vectors
        // TODO: extend for the control sources

        // 1. Calculate pileup for the signal & control sources
        let args = (&query.0, query.1, query.2);
        let control = self.pileup(args.clone(), control)?;
        let signal = self.pileup(args, signal)?;

        // Merge controls using the maximum baseline as a baseline
        // let control_cache: (Vec<Cnts>, Vec<u32>) =
        //     self.rlecache.pop().map(|x| x.into()).unwrap_or_default();
        // let control = rle_vec::merge(&control)
        //     .with_identical(RleIdentical::new(config.sensitivity))
        //     .with_merge(MergeFn::new(
        //         |cnt: &Cnts| cnt.max(config.control_baseline).clone(),
        //         |cnts| {
        //             let mut maximum = config.control_baseline.clone();
        //             for value in cnts {
        //                 if **value > maximum {
        //                     maximum = *value.clone();
        //                 }
        //             }
        //             maximum
        //         },
        //     ))
        //     .save_to(control_cache)
        //     .run()?;

        // 2. Calculate the enrichment
        let mut enrichment = PerOrientation::default();

        for (srle, crle, erle) in izip!(signal.iter(), control.iter(), enrichment.iter_mut()) {
            let cache = self.rle_cache.pop().unwrap_or_default();
            let mut result = rle_vec::merge2(srle, crle)
                .with_identical(RleIdentical::new(config.sensitivity))
                .with_merge2(Merge2Fn::new(
                    |_| unreachable!("This should never be called"),
                    |signal, control| *signal / *control,
                ))
                .save_to(cache)
                .run()?;
            std::mem::swap(erle, &mut result);
        }

        // 3. Call peaks
        let mut peaks = PerOrientation::default();
        let roi_start = query.1;

        for (enrichment, orientation) in enrichment.into_iter() {
            let result = pcalling::run(&enrichment, &config.pcalling)
                .into_iter()
                .map(|x| {
                    let start = Idx::from(*x.start()).unwrap() + roi_start;
                    let end = Idx::from(*x.end()).unwrap() + roi_start;
                    let summit = Idx::from(*x.summit()).unwrap() + roi_start;
                    pcalling::Peak::new(start, end, x.signal().clone(), summit)
                })
                .collect::<Vec<_>>();

            *peaks.get_mut(orientation) = result;
            self.rle_cache.push(enrichment);
        }

        // 4. Save results
        let segment = Segment::new(query.1, query.2)?;
        let result = Region::new(query.0.clone(), segment, peaks);

        match self.comparisons.insert((cmpind, queryind), result) {
            Some(_) => Err(eyre::eyre!(
                "Ripper worker was called twice with the same comparison and query indices. \
                That must not happen and indicates a bug in the code."
            )),
            None => Ok(()),
        }
    }

    pub fn collapse<'a, Tag>(
        comparisons: Vec<Tag>,
        queries: Vec<(Ctg, Idx, Idx)>,
        workers: impl Iterator<Item = &'a mut Worker<Ctg, Idx, Cnts>>,
    ) -> Vec<Ripped<Ctg, Idx, Cnts, Tag>>
    where
        Ctg: 'a,
        Idx: 'a,
        Cnts: 'a,
    {
        let mut buffer = vec![vec![None; queries.len()]; comparisons.len()];

        for worker in workers {
            for ((cmpind, queryind), peaks) in worker.comparisons.drain() {
                assert!(buffer[cmpind][queryind].is_none());
                buffer[cmpind][queryind] = Some(peaks);
            }
        }

        let mut result = Vec::with_capacity(comparisons.len());
        for (tag, buffer) in izip!(comparisons, buffer) {
            let buffer = izip!(&queries, buffer.into_iter())
                .map(|(query, peaks)| {
                    let peaks = peaks.unwrap();
                    assert_eq!(&query.0, peaks.contig());
                    assert_eq!(query.1, peaks.segment().start());
                    assert_eq!(query.2, peaks.segment().end());
                    peaks
                })
                .collect();
            result.push(Ripped::new(tag, buffer));
        }
        result
    }
}
