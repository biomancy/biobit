use crate::worker::RleIdentical;
use biobit_collections_rs::rle_vec::RleVec;
use biobit_core_rs::loc::{
    ChainInterval, Contig, Interval, IntervalOp, Orientation, PerOrientation,
};
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::source::{AnyMap, Source};
use biobit_core_rs::LendingIterator;
use biobit_io_rs::bam::SegmentedAlignment;
use derive_getters::{Dissolve, Getters};
use derive_more::Into;
use eyre::{eyre, OptionExt, Report, Result};
use higher_kinded_types::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Into, Getters)]
pub struct ControlModel<Idx: PrimInt> {
    regions: Vec<ChainInterval<Idx>>,
    uniform_baseline: bool,
    winsizes: Vec<usize>,
    bufsize: usize,
}

impl<Idx: PrimInt> ControlModel<Idx> {
    pub fn new(
        mut regions: Vec<ChainInterval<Idx>>,
        uniform_baseline: bool,
        winsizes: Vec<usize>,
    ) -> Result<Self> {
        let bufsize = regions
            .iter()
            .map(|x| x.links().iter().fold(Idx::zero(), |acc, l| acc + l.len()))
            .max()
            .ok_or_eyre("Control model must have at least one region")?
            .to_usize()
            .ok_or_eyre("Size of the largest chain exceeds usize")?;

        for s in &winsizes {
            if *s <= 1 {
                return Err(eyre!("Smoothing window size must be greater than one"));
            }
        }
        regions.sort();

        Ok(Self {
            regions,
            uniform_baseline,
            winsizes,
            bufsize,
        })
    }

    pub fn apply<Cnts: Float>(
        &self,
        start: Idx,
        end: Idx,
        epsilon: Cnts,
        source: &[Cnts],
        saveto: &mut [Cnts],
    ) -> Result<()> {
        // Prepare the buffer
        let mut buffer = Vec::with_capacity(self.bufsize);
        buffer.clear();

        // Prepare the ROI coordinates and length
        let roi = Interval::new(start, end).unwrap();
        let length = roi.len();
        debug_assert_eq!(length.to_usize().unwrap(), saveto.len());

        // Each chain is processed independently
        let mut links = Vec::new();
        let mut backmap = Vec::new();
        for chain in &self.regions {
            // Remap chain links to local coordinates
            links.extend(
                chain
                    .links()
                    .iter()
                    .filter_map(|x| (x << start).clamped(&roi)),
            );
            if links.is_empty() {
                continue;
            }

            // Populate the buffer with the data from the source array and establish backlinks
            for link in &links {
                let link = link.start().to_usize().unwrap()..link.end().to_usize().unwrap();
                backmap.push(link.clone());
                buffer.extend_from_slice(&source[link]);
            }

            // If the total signal is below the threshold, skip the chain
            let sumval = buffer.iter().fold(Cnts::zero(), |acc, x| acc + *x);
            if sumval < epsilon {
                links.clear();
                backmap.clear();
                buffer.clear();
                continue;
            }

            // Calculate the smoothed signal for the chain using each window size
            for &winsize in &self.winsizes {
                if winsize >= buffer.len() {
                    continue;
                }

                // Initialize the filter
                let mut sum = buffer
                    .iter()
                    .take(winsize)
                    .fold(Cnts::zero(), |acc, x| acc + *x);

                // Backtracking utilities
                let offset = winsize / 2;
                let backpos = backmap.iter().flat_map(|x| x.clone()).skip(offset);
                let bufpos = offset..buffer.len() - offset;

                // Run the calculations
                for (bufpos, backpos) in bufpos.zip(backpos) {
                    saveto[backpos] = saveto[backpos].max(sum / Cnts::from(winsize).unwrap());
                    sum = sum + buffer[bufpos + offset] - buffer[bufpos - offset];
                }
            }

            // Apply the uniform baseline if necessary
            if self.uniform_baseline {
                let baseline = sumval / Cnts::from(length).unwrap();
                for link in &links {
                    let rng = link.start().to_usize().unwrap()..link.end().to_usize().unwrap();
                    for val in saveto[rng].iter_mut() {
                        *val = val.max(baseline);
                    }
                }
            }

            // Clear the buffer
            links.clear();
            backmap.clear();
            buffer.clear();
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Dissolve, Getters)]
pub struct RNAPileup<Idx: PrimInt, Cnts: Float> {
    sensitivity: Cnts,
    min_signal: Cnts,
    control_baseline: Cnts,
    buffer: Vec<Cnts>,
    modeling: PerOrientation<Vec<ControlModel<Idx>>>,
}

impl<Idx: PrimInt, Cnts: Float> Default for RNAPileup<Idx, Cnts> {
    fn default() -> Self {
        RNAPileup {
            sensitivity: Cnts::from(Self::DEFAULT_SENSITIVITY).unwrap(),
            control_baseline: Cnts::zero(),
            min_signal: Cnts::zero(),
            buffer: Vec::new(),
            modeling: PerOrientation::default(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float> RNAPileup<Idx, Cnts> {
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

    pub fn add_modeling(
        &mut self,
        orientation: Orientation,
        model: ControlModel<Idx>,
    ) -> &mut Self {
        self.modeling.get_mut(orientation).push(model);
        self
    }

    pub fn identical(&self) -> RleIdentical<Cnts> {
        RleIdentical::new(self.sensitivity)
    }

    #[allow(clippy::type_complexity)]
    fn pileup<Ctg, Src>(
        &self,
        query: (&Ctg, Idx, Idx),
        sources: &mut [Src],
        caches: &mut AnyMap,
        mut counts: PerOrientation<Vec<Cnts>>,
    ) -> Result<PerOrientation<Vec<Cnts>>>
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
        Ok(counts)
    }

    fn rlencode(
        &self,
        counts: &PerOrientation<Vec<Cnts>>,
        cache: PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>,
    ) -> Result<PerOrientation<RleVec<Cnts, u32, RleIdentical<Cnts>>>> {
        cache.try_map::<_, Report>(|o, rle| {
            let cnts = counts.get(o);
            let rle = rle
                .rebuild()
                .with_identical(self.identical())
                .with_dense_values(cnts)?
                .build();

            Ok(rle)
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn model_signal<Ctg, Src>(
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
        let counts = self.pileup(query, sources, caches, counts)?;
        let mut rle = self.rlencode(&counts, rle)?;

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
    pub fn model_control<Ctg, Src>(
        &self,
        query: (&Ctg, Idx, Idx),
        sources: &mut [Src],
        caches: &mut AnyMap,
        counts: PerOrientation<Vec<Cnts>>,
        model: &mut PerOrientation<Vec<Cnts>>,
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
        let counts = self.pileup(query, sources, caches, counts)?;
        // Build the control model
        model.try_apply(|o, x| {
            x.clear();
            x.extend_from_slice(counts.get(o));

            for m in self.modeling.get(o) {
                m.apply(query.1, query.2, self.sensitivity, counts.get(o), x)?;
            }
            Ok::<(), Report>(())
        })?;

        // Run-length encode the control model
        let mut rle = self.rlencode(model, rle)?;
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
