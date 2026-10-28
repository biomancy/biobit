use super::excess_profile::ExcessProfile;
use crate::result::Peak;
use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::loc::mapping::{ChainMap, Mapping};
use biobit_core_rs::loc::{ChainInterval, Interval, IntervalOp, Orientation, PerOrientation};
use biobit_core_rs::num::{Float, PrimInt, PrimUInt};
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::Getters;
use derive_more::Into;
use eyre::{OptionExt, Result, eyre};
use std::ops::Range;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Into, Debug)]
pub struct NMSRegions<Idx: PrimInt> {
    pub regions: Vec<ChainInterval<Idx>>,
    pub uniform_baseline: bool,
}

impl<Idx: PrimInt> NMSRegions<Idx> {
    pub fn new(mut regions: Vec<ChainInterval<Idx>>, uniform_baseline: bool) -> Result<Self> {
        if regions.is_empty() {
            return Err(eyre!("NMS regions must not be empty"));
        }
        regions.sort();

        Ok(Self {
            regions,
            uniform_baseline,
        })
    }
}

/// Retains candidate peak positions whose positive excess (max(0, scaled_signal - scaled_control))
/// meets a multiple of the local mean excess. The local mean is defined over regions covered by
/// either the signal or control model. That is, zero excess still contributes to the mean's denominator.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Into, Getters)]
pub struct NMS<Idx: PrimInt, Cnts> {
    fecutoff: Cnts,
    group_within: Idx,
    slopfrac: f32,
    sloplim: (Idx, Idx),
    roi: PerOrientation<Vec<NMSRegions<Idx>>>,
}

impl<Idx: PrimInt, Cnts: Float> Default for NMS<Idx, Cnts> {
    fn default() -> Self {
        NMS {
            fecutoff: Cnts::one(),
            group_within: Idx::zero(),
            slopfrac: 1.0,
            sloplim: (Idx::zero(), Idx::max_value()),
            roi: PerOrientation::default(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float> NMS<Idx, Cnts> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_fecutoff(&mut self, fecutoff: Cnts) -> Result<&mut Self> {
        if !fecutoff.is_finite() || fecutoff < Cnts::one() {
            return Err(eyre!(
                "NMS fold-enrichment cutoff must be finite and at least one"
            ));
        }

        self.fecutoff = fecutoff;
        Ok(self)
    }

    pub fn set_group_within(&mut self, group_within: Idx) -> Result<&mut Self> {
        if group_within < Idx::zero() {
            return Err(eyre!("NMS grouping distance must be non-negative"));
        }

        self.group_within = group_within;
        Ok(self)
    }

    pub fn set_slopfrac(&mut self, slopfrac: f32) -> Result<&mut Self> {
        if !slopfrac.is_finite() || slopfrac < 0.0 {
            return Err(eyre!("NMS slop fraction must be finite and non-negative"));
        }

        self.slopfrac = slopfrac;
        Ok(self)
    }

    pub fn set_sloplim(&mut self, minslop: Idx, maxslop: Idx) -> Result<&mut Self> {
        if minslop > maxslop {
            return Err(eyre!("Minimum NMS slop must not exceed maximum slop"));
        } else if minslop < Idx::zero() {
            return Err(eyre!("Minimum NMS slop must be non-negative"));
        }

        self.sloplim = (minslop, maxslop);
        Ok(self)
    }

    pub fn add_regions(&mut self, orientation: Orientation, regions: NMSRegions<Idx>) -> &mut Self {
        self.roi[orientation].push(regions);
        self
    }

    fn threshold<Len: PrimUInt>(
        &self,
        nms: &[Interval<Idx>],
        profile: &ExcessProfile<Cnts, Len>,
    ) -> Result<Option<Cnts>> {
        let mut total = 0.0f64;
        let mut covered = 0usize;
        for interval in nms {
            let query = Range::from(interval.cast::<usize>().unwrap());
            for (overlap, excess) in profile.covered_runs(query) {
                let width = overlap.end - overlap.start;
                total += excess
                    .to_f64()
                    .ok_or_eyre("Failed to convert the NMS excess to f64")?
                    * width as f64;
                covered += width;
            }
        }
        // Normally, enriched candidates contribute covered bases. Coverage can
        // still be absent if ROI clipping retains only uncovered parts of a
        // candidate, or the candidates were derived from different model profiles.
        if covered == 0 {
            return Ok(None);
        }
        let baseline = Cnts::from(total / covered as f64)
            .ok_or_eyre("Failed to convert the NMS baseline to the count type")?;
        if !baseline.is_finite() {
            return Err(eyre!("NMS baseline is not finite"));
        }

        // Excess is non-negative, so a zero mean means the covered background
        // has no positive excess (apart from numerical underflow). This should
        // not happen if it includes positive-excess bases from the candidates,
        // but ROI clipping or different model profiles can break that assumption.
        // Without a positive reference, return no threshold and skip quietly.
        if baseline <= Cnts::zero() {
            return Ok(None);
        }
        let cutoff = baseline * self.fecutoff;
        if !cutoff.is_finite() {
            return Err(eyre!("NMS cutoff is not finite"));
        }
        Ok(Some(cutoff))
    }

    fn filter_intervals<Len: PrimUInt>(
        cutoff: Cnts,
        intervals: &[Interval<Idx>],
        profile: &ExcessProfile<Cnts, Len>,
        saveto: &mut Vec<Peak<Idx, Cnts>>,
    ) {
        for interval in intervals {
            let query = Range::from(interval.cast::<usize>().unwrap());
            let mut current: Option<(Idx, Idx, Cnts, Idx)> = None;
            for (overlap, value) in profile.covered_runs(query) {
                if value < cutoff {
                    continue;
                }
                let start = Idx::from(overlap.start).unwrap();
                let end = Idx::from(overlap.end).unwrap();
                if let Some(peak) = current.as_mut() {
                    if peak.1 == start {
                        peak.1 = end;
                        if value > peak.2 {
                            peak.2 = value;
                            peak.3 = start;
                        }
                        // Fast path: proceed to the next overlap directly
                        continue;
                    }
                    saveto.push(Peak::new(peak.0, peak.1, peak.2, peak.3).unwrap());
                }

                current = Some((start, end, value, start));
            }
            if let Some(peak) = current {
                saveto.push(Peak::new(peak.0, peak.1, peak.2, peak.3).unwrap());
            }
        }
    }

    fn nmsit_uniform<Len: PrimUInt>(
        &self,
        nms: &[Interval<Idx>],
        profile: &ExcessProfile<Cnts, Len>,
        peaks: &[Peak<Idx, Cnts>],
        saveto: &mut Vec<Peak<Idx, Cnts>>,
    ) -> Result<()> {
        debug_assert!(!peaks.is_empty());
        debug_assert!(!nms.is_empty());

        let mut overlapping = Vec::with_capacity(peaks.len());
        for peak in peaks {
            if peak.interval().start() >= nms.last().unwrap().end() {
                break;
            } else if peak.interval().end() <= nms.first().unwrap().start() {
                continue;
            }

            for nms in nms {
                if nms.start() >= peak.interval().end() {
                    break;
                } else if nms.end() <= peak.interval().start() {
                    continue;
                }

                if let Some(intersection) = peak.interval().intersection(nms) {
                    overlapping.push(intersection);
                }
            }
        }
        if overlapping.is_empty() {
            return Ok(());
        }

        let Some(cutoff) = self.threshold(nms, profile)? else {
            return Ok(());
        };

        Self::filter_intervals(cutoff, &overlapping, profile, saveto);
        Ok(())
    }

    fn nmsit_slop<Len: PrimUInt>(
        &self,
        nms: &[Interval<Idx>],
        profile: &ExcessProfile<Cnts, Len>,
        peaks: &[Peak<Idx, Cnts>],
        saveto: &mut Vec<Peak<Idx, Cnts>>,
    ) -> Result<()> {
        debug_assert!(!peaks.is_empty());
        debug_assert!(!nms.is_empty());

        let mapping = ChainMap::new(ChainInterval::try_from_iter(nms.iter().copied())?);

        let mut mapped = Vec::new();
        for peak in peaks {
            if peak.interval().start() >= nms.last().unwrap().end() {
                break;
            } else if peak.interval().end() <= nms.first().unwrap().start() {
                continue;
            }
            match mapping.map_interval(peak.interval()) {
                Mapping::Complete(interval) | Mapping::Truncated(interval) => mapped.push(interval),
                Mapping::None => continue,
            }
        }
        if mapped.is_empty() {
            return Ok(());
        }

        let groups = group_within(&mapped, self.group_within);
        let mut buffer = ChainInterval::default();
        for group in groups {
            let start = group.first().unwrap().start();
            let end = group.last().unwrap().end();

            let scaled_slop = (end - start)
                .to_f64()
                .ok_or_eyre("Failed to convert NMS group length to f64")?
                * f64::from(self.slopfrac);
            let minslop = self.sloplim.0.to_f64().unwrap();
            let maxslop = self.sloplim.1.to_f64().unwrap();
            let slop = if scaled_slop <= minslop {
                self.sloplim.0
            } else if scaled_slop >= maxslop {
                self.sloplim.1
            } else {
                Idx::from(scaled_slop).unwrap()
            };

            let query = Interval::new(start.saturating_sub(slop), end.saturating_add(slop))?;
            let chain = match mapping.invmap_interval(&query, std::mem::take(&mut buffer)) {
                Mapping::Complete(chain) | Mapping::Truncated(chain) => chain,
                Mapping::None => return Err(eyre!("Failed to map an NMS background region")),
            };

            let threshold = self.threshold(chain.links(), profile)?;
            buffer = chain;
            let Some(cutoff) = threshold else {
                continue;
            };

            for peak in group {
                let chain = match mapping.invmap_interval(peak, std::mem::take(&mut buffer)) {
                    Mapping::Complete(chain) | Mapping::Truncated(chain) => chain,
                    Mapping::None => return Err(eyre!("Failed to map an NMS peak region")),
                };
                Self::filter_intervals(cutoff, chain.links(), profile, saveto);
                buffer = chain;
            }
        }
        Ok(())
    }

    fn merge_allowed(mut allowed: Vec<Peak<Idx, Cnts>>) -> Vec<Peak<Idx, Cnts>> {
        allowed.sort_by_key(|peak| *peak.interval());
        let mut result = Vec::with_capacity(allowed.len());
        let mut iterator = allowed.into_iter();
        let first = if let Some(first) = iterator.next() {
            first
        } else {
            return result;
        };

        let mut current = (
            first.interval().start(),
            first.interval().end(),
            *first.signal(),
            *first.summit(),
        );

        for peak in iterator {
            if peak.interval().start() <= current.1 {
                current.1 = current.1.max(peak.interval().end());
                if *peak.signal() > current.2 {
                    current.2 = *peak.signal();
                    current.3 = *peak.summit();
                }
            } else {
                result.push(Peak::new(current.0, current.1, current.2, current.3).unwrap());
                current = (
                    peak.interval().start(),
                    peak.interval().end(),
                    *peak.signal(),
                    *peak.summit(),
                );
            }
        }
        result.push(Peak::new(current.0, current.1, current.2, current.3).unwrap());
        result
    }

    /// Filters candidates using the provided normalized signal and control models.
    pub fn run<Len: PrimUInt, I: Identical<Cnts>>(
        &self,
        orientation: Orientation,
        region: (Idx, Idx),
        peaks: &[Peak<Idx, Cnts>],
        signal: &RleVec<Cnts, Len, I>,
        control: &RleVec<Cnts, Len, I>,
    ) -> Result<Vec<Peak<Idx, Cnts>>> {
        if peaks.is_empty() || self.roi[orientation].is_empty() {
            return Ok(Vec::new());
        }

        let region = Interval::new(region.0, region.1)?;
        let length = region
            .len()
            .to_usize()
            .ok_or_eyre("NMS region length exceeds usize")?;
        let profile = ExcessProfile::new(signal, control, length)?;
        debug_assert!(peaks.is_sorted_by_key(|peak| peak.interval().start()));

        let local_end = region.len();
        debug_assert!(
            !peaks
                .iter()
                .any(|peak| peak.interval().start() < Idx::zero()
                    || peak.interval().end() > local_end)
        );

        let mut allowed = Vec::with_capacity(peaks.len());
        for config in &self.roi[orientation] {
            for nms in &config.regions {
                let nms = nms
                    .links()
                    .iter()
                    .filter_map(|interval| interval.clamped(&region))
                    .map(|interval| interval << region.start())
                    .collect::<Vec<_>>();
                if nms.is_empty() {
                    continue;
                }

                let start = match peaks
                    .binary_search_by_key(&nms[0].start(), |peak| peak.interval().start())
                {
                    Ok(index) => index,
                    Err(0) => 0,
                    Err(index) => index - 1,
                };

                if config.uniform_baseline {
                    self.nmsit_uniform(&nms, &profile, &peaks[start..], &mut allowed)?;
                } else {
                    self.nmsit_slop(&nms, &profile, &peaks[start..], &mut allowed)?;
                }
            }
        }

        Ok(Self::merge_allowed(allowed))
    }
}

fn group_within<Idx: PrimInt>(
    mut intervals: &[Interval<Idx>],
    group_within: Idx,
) -> impl Iterator<Item = &[Interval<Idx>]> {
    std::iter::from_fn(move || {
        if intervals.is_empty() {
            return None;
        }

        let mut end = 1;
        while end < intervals.len() {
            debug_assert!(intervals[end].start() >= intervals[end - 1].end());
            if intervals[end].start() - intervals[end - 1].end() > group_within {
                break;
            }
            end += 1;
        }

        let (group, remaining) = intervals.split_at(end);
        intervals = remaining;
        Some(group)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestRle = RleVec<f64, u32, fn(&f64, &f64) -> bool>;

    fn rle(values: &[f64]) -> TestRle {
        RleVec::builder((|a: &f64, b: &f64| a == b) as fn(&f64, &f64) -> bool)
            .with_dense_values(values)
            .unwrap()
            .build()
    }

    fn chain(start: usize, end: usize) -> Result<ChainInterval<usize>> {
        ChainInterval::try_from_iter([Interval::new(start, end)?].into_iter())
    }

    fn peak(start: usize, end: usize, signal: f64, summit: usize) -> Peak<usize, f64> {
        Peak::new(start, end, signal, summit).unwrap()
    }

    #[test]
    fn run_weighted_thresholds_and_peaks_match_a_dense_reference() -> Result<()> {
        use crate::pcalling::ByCutoff;

        let mut nms = NMS::<usize, f64>::new();
        nms.set_fecutoff(1.5)?;
        // Exhaust all pairs of four-base profiles over three coverage levels.
        // This covers independent run boundaries, uncovered gaps, covered zeros,
        // clipping inside runs, and first-maximum summit selection.
        for mut code in 0..3usize.pow(8) {
            let mut values = [0.0; 8];
            for value in &mut values {
                *value = [0.0, 0.5, 3.0][code % 3];
                code /= 3;
            }
            let (signal, control) = values.split_at(4);
            let (s, c) = (rle(signal), rle(control));
            let profile = ExcessProfile::new(&s, &c, signal.len())?;
            for start in 0..4 {
                for end in start + 1..=4 {
                    let interval = Interval::new(start, end)?;
                    let mut total = 0.0;
                    let mut covered = 0;
                    for pos in start..end {
                        if signal[pos] > 0.0 || control[pos] > 0.0 {
                            total += (signal[pos] - control[pos]).max(0.0);
                            covered += 1;
                        }
                    }
                    let expected = if total > 0.0 {
                        Some(total / covered as f64 * 1.5)
                    } else {
                        None
                    };
                    assert_eq!(nms.threshold(&[interval], &profile)?, expected);
                    if let Some(cutoff) = expected {
                        let caller = ByCutoff {
                            min_length: 1,
                            merge_within: 0,
                            cutoff,
                        };
                        let mut dense = Vec::new();
                        caller.run_from_iter(
                            (start..end)
                                .map(|pos| (pos, pos + 1, (signal[pos] - control[pos]).max(0.0))),
                            &mut dense,
                        );
                        let mut compressed = Vec::new();
                        NMS::filter_intervals(cutoff, &[interval], &profile, &mut compressed);
                        assert_eq!(
                            compressed, dense,
                            "signal={signal:?}, control={control:?}, interval={interval:?}"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn indexed_runs_preserve_chained_roi_and_slop_results() -> Result<()> {
        let chain = ChainInterval::try_from_iter(
            [Interval::new(101, 105)?, Interval::new(108, 112)?].into_iter(),
        )?;
        let signal = rle(&[
            0.0, 4.0, 4.0, 2.0, 7.0, 0.0, 0.0, 0.0, 5.0, 5.0, 8.0, 1.0, 0.0, 0.0,
        ]);
        let control = rle(&[
            0.0, 1.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 1.0, 0.0, 0.0,
        ]);
        let peaks = [peak(0, 6, 7.0, 4), peak(7, 14, 8.0, 10)];
        for uniform in [false, true] {
            let mut nms = NMS::new();
            nms.set_sloplim(100, 100)?;
            nms.add_regions(
                Orientation::Forward,
                NMSRegions::new(vec![chain.clone()], uniform)?,
            );
            assert_eq!(
                nms.run(Orientation::Forward, (100, 114), &peaks, &signal, &control)?,
                vec![
                    peak(4, 5, 6.0, 4),
                    peak(8, 9, 4.0, 8),
                    peak(10, 11, 6.0, 10)
                ],
            );
        }
        Ok(())
    }

    #[test]
    fn constant_profiles_preserve_cutoffs_without_dense_expansion() -> Result<()> {
        // Even a billion logical bases occupy one run; no per-base work is needed.
        for length in [4, 1_000_000_000usize] {
            for uniform in [false, true] {
                for (signal, control, cutoff, expected) in [
                    (3.0, 1.0, 1.0, Some(2.0)),
                    (3.0, 1.0, 2.0, None),
                    (0.0, 0.0, 1.0, None), // Uncovered background.
                    (1.0, 1.0, 1.0, None), // Covered, zero excess.
                    (1.0, 2.0, 1.0, None), // Negative residual is clipped to zero.
                ] {
                    let (mut s, mut c) = (rle(&[]), rle(&[]));
                    s.push(signal, length as u32);
                    c.push(control, length as u32);
                    let mut nms = NMS::new();
                    nms.set_fecutoff(cutoff)?;
                    nms.add_regions(
                        Orientation::Forward,
                        NMSRegions::new(vec![chain(0, length)?], uniform)?,
                    );
                    assert_eq!(
                        nms.run(
                            Orientation::Forward,
                            (0, length),
                            &[peak(0, length, 3.0, 0)],
                            &s,
                            &c,
                        )?,
                        expected
                            .map(|excess| peak(0, length, excess, 0))
                            .into_iter()
                            .collect::<Vec<_>>(),
                        "length={length}, uniform={uniform}, signal={signal}, control={control}, cutoff={cutoff}",
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn rejects_mismatched_profile_lengths() -> Result<()> {
        let mut nms = NMS::new();
        nms.add_regions(
            Orientation::Forward,
            NMSRegions::new(vec![chain(0, 5)?], true)?,
        );
        for (signal, control) in [(4, 5), (5, 4), (4, 4), (6, 6), (0, 0)] {
            assert!(
                nms.run(
                    Orientation::Forward,
                    (0, 5),
                    &[peak(0, 5, 3.0, 0)],
                    &rle(&vec![3.0; signal]),
                    &rle(&vec![1.0; control]),
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn configured_nms_uses_local_contrast_and_deduplicates_regions() -> Result<()> {
        let mut nms = NMS::<usize, f64>::new();
        nms.set_fecutoff(2.0)?;
        let regions = NMSRegions::new(vec![chain(100, 106)?], true)?;
        nms.add_regions(Orientation::Forward, regions.clone())
            .add_regions(Orientation::Forward, regions);

        let signal = [1.0, 1.0, 6.0, 6.0, 1.0, 1.0];
        let control = [1.0; 6];
        let result = nms.run(
            Orientation::Forward,
            (100, 106),
            &[peak(0, 6, 6.0, 2)],
            &rle(&signal),
            &rle(&control),
        )?;

        assert_eq!(result, vec![peak(2, 4, 5.0, 2)]);
        Ok(())
    }

    #[test]
    fn slop_baseline_is_divided_by_the_number_of_covered_bases() -> Result<()> {
        let mut nms = NMS::<usize, f64>::new();
        nms.set_fecutoff(1.5)?
            .set_slopfrac(1.0)?
            .set_sloplim(0, 10)?;
        nms.add_regions(
            Orientation::Forward,
            NMSRegions::new(vec![chain(0, 10)?], false)?,
        );

        let signal = [1.0, 1.0, 1.0, 1.0, 4.0, 4.0, 1.0, 1.0, 1.0, 1.0];
        let control = [1.0; 10];
        let result = nms.run(
            Orientation::Forward,
            (0, 10),
            &[peak(4, 6, 4.0, 4)],
            &rle(&signal),
            &rle(&control),
        )?;

        assert_eq!(result, vec![peak(4, 6, 3.0, 4)]);
        Ok(())
    }

    #[test]
    fn slop_is_clamped_before_index_conversion() -> Result<()> {
        let mut nms = NMS::<u8, f64>::new();
        nms.set_slopfrac(f32::MAX)?.set_sloplim(0, 3)?;
        nms.add_regions(
            Orientation::Forward,
            NMSRegions::new(
                vec![ChainInterval::try_from_iter(
                    [Interval::new(0, 20)?].into_iter(),
                )?],
                false,
            )?,
        );

        let mut signal = vec![1.0; 20];
        signal[10] = 4.0;
        let control = vec![1.0; 20];
        let result = nms.run(
            Orientation::Forward,
            (0, 20),
            &[Peak::new(10, 11, 4.0, 10)?],
            &rle(&signal),
            &rle(&control),
        )?;

        assert_eq!(result, vec![Peak::new(10, 11, 3.0, 10)?]);
        Ok(())
    }

    #[test]
    fn only_configured_orientations_are_eligible() -> Result<()> {
        let (signal, control) = (rle(&[3.0; 2]), rle(&[1.0; 2]));
        for configured in [false, true] {
            let mut nms = NMS::new();
            if configured {
                nms.add_regions(
                    Orientation::Forward,
                    NMSRegions::new(vec![chain(0, 2)?], true)?,
                );
            }
            for orientation in [Orientation::Forward, Orientation::Reverse] {
                let expected = if configured && orientation == Orientation::Forward {
                    vec![peak(0, 2, 2.0, 0)]
                } else {
                    vec![]
                };
                assert_eq!(
                    nms.run(
                        orientation,
                        (0, 2),
                        &[peak(0, 2, 3.0, 0)],
                        &signal,
                        &control
                    )?,
                    expected,
                );
            }
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_numerical_configuration() {
        let mut nms = NMS::<i64, f64>::new();
        assert!(nms.set_fecutoff(f64::NAN).is_err());
        assert!(nms.set_fecutoff(f64::INFINITY).is_err());
        assert!(nms.set_slopfrac(f32::NAN).is_err());
        assert!(nms.set_slopfrac(f32::INFINITY).is_err());
        assert!(nms.set_sloplim(-1, 1).is_err());
        assert!(nms.set_sloplim(2, 1).is_err());
    }
}
