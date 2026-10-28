use super::Scaling;
use crate::worker::RleIdentical;
use biobit_collections_rs::rle_vec::RleVec;
use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{
    ChainInterval, Contig, Interval, IntervalOp, Orientation, PerOrientation,
};
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::source::{AnyMap, Source};
use biobit_io_rs::bam::SegmentedAlignment;
use derive_getters::{Dissolve, Getters};
use derive_more::Into;
use eyre::{OptionExt, Report, Result, eyre};
use higher_kinded_types::prelude::*;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug, Into, Getters, Dissolve)]
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
        let roi = Interval::new(start, end)?;
        let length = roi
            .len()
            .to_usize()
            .ok_or_eyre("Control-model query length exceeds usize")?;
        if source.len() != length || saveto.len() != length {
            return Err(eyre!(
                "Control-model buffers must match the query length ({length}), got source={} and destination={}",
                source.len(),
                saveto.len()
            ));
        }

        // Each chain is processed independently
        let mut links = Vec::new();
        let mut backmap = Vec::new();
        for chain in &self.regions {
            // Remap chain links to local coordinates
            links.extend(
                chain
                    .links()
                    .iter()
                    .filter_map(|x| x.clamped(&roi).map(|x| x << start)),
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
                if winsize > buffer.len() {
                    continue;
                }

                // Initialize the filter
                let mut sum = buffer
                    .iter()
                    .take(winsize)
                    .fold(Cnts::zero(), |acc, x| acc + *x);

                let window_count = buffer.len() - winsize + 1;
                let mut centers = backmap
                    .iter()
                    .flat_map(|range| range.clone())
                    .skip(winsize / 2);
                let divisor = Cnts::from(winsize).unwrap();

                // Emit the first window, then roll the sum through the remaining windows.
                let center = centers.next().unwrap();
                saveto[center] = saveto[center].max(sum / divisor);
                for (window_start, center) in (1..window_count).zip(centers) {
                    sum = sum + buffer[window_start + winsize - 1] - buffer[window_start - 1];
                    saveto[center] = saveto[center].max(sum / divisor);
                }
            }

            // Apply the uniform baseline if necessary
            if self.uniform_baseline {
                let baseline = sumval / Cnts::from(buffer.len()).unwrap();
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

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug, Dissolve, Getters)]
pub struct RNAPileup<Idx: PrimInt, Cnts: Float> {
    scaling: Scaling<Cnts>,
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
            scaling: Scaling::default(),
            control_baseline: Cnts::zero(),
            min_signal: Cnts::one(),
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

    /// Sets scaling applied to RLE values after signal thresholds and the control baseline.
    pub fn set_scaling(&mut self, signal: Cnts, control: Cnts) -> Result<&mut Self> {
        self.scaling = Scaling::new(signal, control)?;
        Ok(self)
    }

    /// Sets the absolute numerical tolerance (finite and positive; default: `1e-6`).
    /// It is used unchanged, independently of signal and control scaling:
    /// - RLE encoding treats raw coverage values as identical when their absolute
    ///   difference is at most sensitivity.
    /// - Control modeling skips a chain when its summed raw per-base coverage
    ///   within the query is below sensitivity.
    /// - Reaper uses sensitivity as the absolute RLE tolerance for dimensionless
    ///   enrichment ratios.
    pub fn set_sensitivity(&mut self, sensitivity: Cnts) -> Result<&mut Self> {
        if !sensitivity.is_finite() || sensitivity <= Cnts::zero() {
            return Err(eyre!("Sensitivity must be finite and greater than zero"));
        }
        self.sensitivity = sensitivity;
        Ok(self)
    }

    /// Sets the control baseline in raw coverage, before scaling.
    pub fn set_control_baseline(&mut self, control_baseline: Cnts) -> Result<&mut Self> {
        if !control_baseline.is_finite() || control_baseline < Cnts::zero() {
            return Err(eyre!("Control baseline must be finite and non-negative"));
        }
        self.control_baseline = control_baseline;
        Ok(self)
    }

    /// Sets the minimum raw coverage for signal, finite and at least one (default: one).
    pub fn set_min_signal(&mut self, min_signal: Cnts) -> Result<&mut Self> {
        if !min_signal.is_finite() || min_signal < Cnts::one() {
            return Err(eyre!("Minimum signal must be finite and at least one"));
        }
        self.min_signal = min_signal;
        Ok(self)
    }

    pub fn add_control_model(
        &mut self,
        orientation: Orientation,
        model: ControlModel<Idx>,
    ) -> &mut Self {
        self.modeling[orientation].push(model);
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
        let interval = Interval::new(query.1, query.2)?
            .cast::<usize>()
            .ok_or_eyre("Pileup query coordinates must fit usize")?;
        let (start, end) = (query.1, query.2);

        counts.apply(|_, x| {
            x.clear();
            x.resize(interval.len(), Cnts::zero());
        });

        for src in sources {
            src.populate_caches(caches);
            {
                let mut iter = src.fetch(query)?;
                while let Some(blocks) = iter.next() {
                    for (intervals, orientation, n) in blocks?.iter() {
                        let weight = Cnts::one() / Cnts::from(n).unwrap();
                        let saveto = &mut counts[orientation];

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
            let cnts = &counts[o];
            let rle = rle
                .rebuild()
                .with_identical(self.identical())
                .with_dense_values(cnts)?
                .build();

            Ok(rle)
        })
    }

    /// Returns the unscaled dense buffer for reuse, scaled signal RLE, and coverage intervals.
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

        // Threshold raw signal before scaling the retained values.
        let scaling = *self.scaling.signal();
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
                if *val < self.min_signal {
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

                *val = *val * scaling;
                if !val.is_finite() {
                    return Err(eyre!("Scaled pileup counts must be finite"));
                }
                start = end;
            }

            // Save the model & covered segments
            if covered_end != query.1 {
                covered_cache.push(Interval::new(covered_start, covered_end)?);
            }
            covered[o] = covered_cache;

            if model_end != query.1 {
                model_cache.push(Interval::new(model_start, model_end)?);
            }
            modeled[o] = model_cache;

            Ok::<(), Report>(())
        })?;

        Ok((counts, rle, covered, modeled))
    }

    /// Returns the unscaled dense buffer for reuse, scaled control RLE, and coverage intervals.
    /// The dense model buffer is also kept in unscaled units.
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
        let scaling = *self.scaling.control();
        let counts = self.pileup(query, sources, caches, counts)?;
        // Build the control model
        model.try_apply(|o, x| {
            x.clear();
            x.extend_from_slice(&counts[o]);

            for m in &self.modeling[o] {
                m.apply(query.1, query.2, self.sensitivity, &counts[o], x)?;
            }
            Ok::<(), Report>(())
        })?;

        // Run-length encode the control model
        let mut rle = self.rlencode(model, rle)?;
        let mut covered: PerOrientation<Vec<_>> = PerOrientation::default();

        // Record raw coverage, apply the control baseline, then scale.
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
                *val = val.max(self.control_baseline) * scaling;
                if !val.is_finite() {
                    return Err(eyre!("Scaled pileup counts must be finite"));
                }
            }

            if covered_end != query.1 {
                covered_cache.push(Interval::new(covered_start, covered_end)?);
            }
            covered[o] = covered_cache;

            Ok::<(), Report>(())
        })?;

        Ok((counts, rle, covered))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothing_an_exact_window() -> Result<()> {
        let region = ChainInterval::<usize>::try_from(vec![(100, 103)])?;
        let model = ControlModel::new(vec![region], false, vec![3])?;
        let mut result = [0.0; 3];
        model.apply(100, 103, 1e-6, &[1.0, 2.0, 3.0], &mut result)?;
        assert_eq!(result, [0.0, 2.0, 0.0]);
        Ok(())
    }

    #[test]
    fn smoothing_a_rolling_window() -> Result<()> {
        let region = ChainInterval::<usize>::try_from(vec![(0, 5)])?;
        let model = ControlModel::new(vec![region], false, vec![3])?;
        let mut result = [0.0; 5];
        model.apply(0, 5, 1e-6, &[1.0, 2.0, 3.0, 4.0, 5.0], &mut result)?;
        assert_eq!(result, [0.0, 2.0, 3.0, 4.0, 0.0]);
        Ok(())
    }

    #[test]
    fn uniform_baseline_uses_chain_length() -> Result<()> {
        let region = ChainInterval::<usize>::try_from(vec![(1, 3)])?;
        let model = ControlModel::new(vec![region], true, vec![])?;
        let mut result = [0.0; 4];
        model.apply(0, 4, 1e-6, &[0.0, 2.0, 4.0, 0.0], &mut result)?;
        assert_eq!(result, [0.0, 3.0, 3.0, 0.0]);
        Ok(())
    }

    #[test]
    fn setters_reject_invalid_model_parameters() {
        let mut model = RNAPileup::<usize, f64>::new();
        assert!(model.set_sensitivity(0.0).is_err());
        assert!(model.set_control_baseline(-1.0).is_err());
        assert_eq!(*model.min_signal(), 1.0);
        assert!(model.set_min_signal(0.5).is_err());
        assert!(model.set_min_signal(1.0).is_ok());
    }
}
