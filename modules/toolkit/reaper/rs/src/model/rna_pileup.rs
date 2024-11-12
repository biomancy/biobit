use ::higher_kinded_types::prelude::*;
use derive_getters::{Dissolve, Getters};
use eyre::{Report, Result};

use biobit_collections_rs::rle_vec::RleVec;
use biobit_core_rs::loc::IntervalOp;
use biobit_core_rs::loc::{Contig, Interval, PerOrientation};
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::source::{AnyMap, Source};
use biobit_core_rs::LendingIterator;
use biobit_io_rs::bam::SegmentedAlignment;

use crate::worker::RleIdentical;

#[derive(Clone, PartialEq, Eq, Debug, Dissolve, Getters)]
pub struct RNAPileup<Cnts: Float> {
    sensitivity: Cnts,
    control_baseline: Cnts,
    min_signal: Cnts,
}

impl<Cnts: Float> Default for RNAPileup<Cnts> {
    fn default() -> Self {
        RNAPileup {
            sensitivity: Cnts::from(Self::DEFAULT_SENSITIVITY).unwrap(),
            control_baseline: Cnts::zero(),
            min_signal: Cnts::zero(),
        }
    }
}

impl<Cnts: Float> RNAPileup<Cnts> {
    pub const DEFAULT_SENSITIVITY: f64 = 1e-6;
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_sensitivity(&mut self, sensitivity: Cnts) -> &mut Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn set_control_baseline(&mut self, control_baseline: Cnts) -> &mut Self {
        self.control_baseline = control_baseline;
        self
    }

    pub fn set_min_signal(&mut self, min_signal: Cnts) -> &mut Self {
        self.min_signal = min_signal;
        self
    }

    pub fn identical(&self) -> RleIdentical<Cnts> {
        RleIdentical::new(self.sensitivity)
    }

    #[allow(clippy::type_complexity)]
    fn pileup<Ctg, Idx, Src>(
        &self,
        query: (&Ctg, Idx, Idx),
        sources: &mut [Src],
        caches: &mut AnyMap,
        mut counts: PerOrientation<Vec<Cnts>>,
        rle: PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
    ) -> Result<(
        PerOrientation<Vec<Cnts>>,
        PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
    )>
    where
        Idx: PrimInt,
        Ctg: Contig,
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        let (start_usize, end_usize) = (query.1.to_usize().unwrap(), query.2.to_usize().unwrap());
        let (start, end) = (query.1, query.2);

        counts.apply(|_, x| {
            x.clear();
            x.resize(end_usize - start_usize, Cnts::zero());
        });

        for src in sources {
            src.populate_caches(caches);
            {
                let mut iter = src.fetch(query)?;
                while let Some(blocks) = iter.next() {
                    for (intervals, orientation, n) in blocks?.iter() {
                        let weight = Cnts::one() / Cnts::from(n).unwrap();
                        let saveto = counts.get_mut(orientation);

                        for s in intervals {
                            if s.end() <= start || s.start() >= end {
                                continue;
                            }

                            // Clip the interval to the query boundaries and transform it to local coordinates
                            let istart = (s.start().max(start) - start).to_usize().unwrap();
                            let iend = (s.end().min(end) - start).to_usize().unwrap();
                            debug_assert!(istart <= iend);

                            for item in &mut saveto[istart..iend] {
                                *item = *item + weight;
                            }
                        }
                    }
                }
            }
            src.release_caches(caches);
        }

        let rle = rle.try_map::<_, Report>(|o, rle| {
            let cnts = counts.get(o);
            let rle = rle
                .rebuild()
                .with_identical(self.identical())
                .with_dense_values(cnts)?
                .build();

            Ok(rle)
        })?;

        Ok((counts, rle))
    }

    #[allow(clippy::type_complexity)]
    pub fn model_signal<Ctg, Idx, Src>(
        &self,
        query: (&Ctg, Idx, Idx),
        sources: &mut [Src],
        caches: &mut AnyMap,
        counts: PerOrientation<Vec<Cnts>>,
        rle: PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
    ) -> Result<(
        PerOrientation<Vec<Cnts>>,
        PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
        PerOrientation<Vec<Interval<Idx>>>,
        PerOrientation<Vec<Interval<Idx>>>,
    )>
    where
        Ctg: Contig,
        Idx: PrimInt,
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        let (counts, mut rle) = self.pileup(query, sources, caches, counts, rle)?;
        let mut covered: PerOrientation<Vec<_>> = PerOrientation::default();
        let mut modeled: PerOrientation<Vec<_>> = PerOrientation::default();

        // Turn signal below the minimum signal into zeros
        rle.try_apply(|o, rle| {
            let mut start = query.1;
            let mut end;

            // Cache for covered / modeled segments
            let (mut covered_start, mut covered_end) = (query.1, query.1);
            let mut covered_cache = Vec::new();

            let (mut model_start, mut model_end) = (query.1, query.1);
            let mut model_cache = Vec::new();

            for (val, length) in rle.runs_mut() {
                end = start + Idx::from(*length).unwrap();

                // Save covered intervals
                if !val.is_zero() {
                    if covered_end == start {
                        covered_end = end;
                    } else {
                        if covered_end != query.1 {
                            covered_cache.push(Interval::new(covered_start, covered_end)?);
                        }
                        covered_start = start;
                        covered_end = end;
                    }
                }

                // Save modeled segments
                if *val <= self.min_signal {
                    *val = Cnts::zero();
                } else if start == model_end {
                    model_end = end;
                } else {
                    if model_end != query.1 {
                        model_cache.push(Interval::new(model_start, model_end)?);
                    }
                    model_start = start;
                    model_end = end;
                }

                start = end;
            }

            // Save the model & covered segments
            if covered_end != query.1 {
                covered_cache.push(Interval::new(covered_start, covered_end)?);
            }
            *covered.get_mut(o) = covered_cache;

            if model_end != query.1 {
                model_cache.push(Interval::new(model_start, model_end)?);
            }
            *modeled.get_mut(o) = model_cache;

            Ok::<(), Report>(())
        })?;

        Ok((counts, rle, covered, modeled))
    }

    #[allow(clippy::type_complexity)]
    pub fn model_control<Ctg, Idx, Src>(
        &self,
        query: (&Ctg, Idx, Idx),
        sources: &mut [Src],
        caches: &mut AnyMap,
        counts: PerOrientation<Vec<Cnts>>,
        rle: PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
    ) -> Result<(
        PerOrientation<Vec<Cnts>>,
        PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
        PerOrientation<Vec<Interval<Idx>>>,
    )>
    where
        Ctg: Contig,
        Idx: PrimInt,
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        let (counts, mut rle) = self.pileup(query, sources, caches, counts, rle)?;
        let mut covered: PerOrientation<Vec<_>> = PerOrientation::default();

        // Turn signal below the minimum signal into zeros
        rle.try_apply(|o, rle| {
            let mut start = query.1;
            let mut end;

            // Cache for covered segments
            let (mut covered_start, mut covered_end) = (query.1, query.1);
            let mut covered_cache = Vec::new();

            for (val, length) in rle.runs_mut() {
                end = start + Idx::from(*length).unwrap();

                // Save covered segments
                if !val.is_zero() {
                    if covered_end == start {
                        covered_end = end;
                    } else {
                        if covered_end != query.1 {
                            covered_cache.push(Interval::new(covered_start, covered_end)?);
                        }
                        covered_start = start;
                        covered_end = end;
                    }
                }

                start = end;
                *val = val.max(self.control_baseline);
            }

            if covered_end != query.1 {
                covered_cache.push(Interval::new(covered_start, covered_end)?);
            }
            *covered.get_mut(o) = covered_cache;

            Ok::<(), Report>(())
        })?;

        Ok((counts, rle, covered))
    }
}
