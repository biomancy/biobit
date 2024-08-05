use ::higher_kinded_types::prelude::*;
use derive_getters::{Dissolve, Getters};
use eyre::{Report, Result};

use biobit_collections_rs::rle_vec::RleVec;
use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{Contig, PerOrientation, Segment};
use biobit_core_rs::loc::AsSegment;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::source::{AnyMap, Source};
use biobit_io_rs::bam::SegmentedAlignment;

use crate::worker::RleIdentical;

#[derive(Clone, PartialEq, Eq, Debug, Dissolve, Getters)]
pub struct RNAPileup<Cnts: Float> {
    sensitivity: Cnts,
    control_baseline: Cnts,
    min_signal: Cnts,
}

impl<Cnts: Float> RNAPileup<Cnts> {
    pub const DEFAULT_SENSITIVITY: f64 = 1e-6;
    pub fn new() -> Self {
        RNAPileup {
            sensitivity: Cnts::from(Self::DEFAULT_SENSITIVITY).unwrap(),
            control_baseline: Cnts::zero(),
            min_signal: Cnts::zero(),
        }
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
                    for (segments, orientation, n) in blocks?.iter() {
                        let weight = Cnts::one() / Cnts::from(n).unwrap();
                        let saveto = counts.get_mut(orientation);

                        for s in segments {
                            if s.end() <= start || s.start() >= end {
                                continue;
                            }

                            // Clip the segment to the query boundaries and transform it to local coordinates
                            let segment_start = (s.start().max(start) - start).to_usize().unwrap();
                            let segment_end = (s.end().min(end) - start).to_usize().unwrap();
                            debug_assert!(segment_start <= segment_end);

                            for i in segment_start..segment_end {
                                saveto[i] = saveto[i] + weight;
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
        PerOrientation<Vec<Segment<Idx>>>,
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
        let mut model: PerOrientation<Vec<_>> = PerOrientation::default();

        // Turn signal below the minimum signal into zeros
        rle.try_apply(|o, rle| {
            let mut start = query.1;
            let mut end;

            // Cache for segments passing the threshold
            let (mut cache_start, mut cache_end) = (query.1, query.1);
            let mut cache = Vec::new();

            for (val, length) in rle.runs_mut() {
                end = start + Idx::from(*length).unwrap();

                if *val < self.min_signal {
                    *val = Cnts::zero();
                } else {
                    if start == cache_end {
                        cache_end = end;
                    } else {
                        if cache_end != query.1 {
                            cache.push(Segment::new(cache_start, cache_end)?);
                        }
                        cache_start = start;
                        cache_end = end;
                    }
                }

                start = end;
            }
            if cache_end != query.1 {
                cache.push(Segment::new(cache_start, cache_end)?);
            }
            *model.get_mut(o) = cache;

            Ok::<(), Report>(())
        })?;

        Ok((counts, rle, model))
    }

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

        // Turn signal below the minimum signal into zeros
        rle.apply(|_, rle| {
            rle.values_mut().for_each(|x| {
                *x = x.max(self.control_baseline);
            });
        });

        Ok((counts, rle))
    }
}
