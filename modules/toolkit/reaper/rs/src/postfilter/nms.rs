use crate::pcalling::ByCutoff;
use crate::result::Peak;
use biobit_core_rs::loc::mapping::{ChainMap, Mapping};
use biobit_core_rs::loc::{ChainInterval, Interval, IntervalOp, Orientation, PerOrientation};
use biobit_core_rs::num::{Float, PrimInt};
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

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Into, Getters)]
pub struct NMS<Idx: PrimInt, Cnts> {
    fecutoff: Cnts,
    group_within: Idx,
    slopfrac: f32,
    sloplim: (Idx, Idx),
    sensitivity: Cnts,
    roi: PerOrientation<Vec<NMSRegions<Idx>>>,
}

impl<Idx: PrimInt, Cnts: Float> Default for NMS<Idx, Cnts> {
    fn default() -> Self {
        NMS {
            fecutoff: Cnts::one(),
            group_within: Idx::zero(),
            slopfrac: 1.0,
            sloplim: (Idx::zero(), Idx::max_value()),
            sensitivity: Cnts::epsilon(),
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

    pub fn set_sensitivity(&mut self, sensitivity: Cnts) -> Result<&mut Self> {
        if !sensitivity.is_finite() || sensitivity <= Cnts::zero() {
            return Err(eyre!(
                "NMS sensitivity must be finite and greater than zero"
            ));
        }

        self.sensitivity = sensitivity;
        Ok(self)
    }

    pub fn add_regions(&mut self, orientation: Orientation, regions: NMSRegions<Idx>) -> &mut Self {
        self.roi[orientation].push(regions);
        self
    }

    fn threshold(
        &self,
        nms: &[Interval<Idx>],
        sigcnts: &[Cnts],
        cntcnts: &[Cnts],
    ) -> Result<Option<(Cnts, Cnts)>> {
        let mut total = 0.0f64;
        let mut minimum = f64::INFINITY;
        let mut covered = 0usize;

        for pos in nms
            .iter()
            .flat_map(|interval| Range::from(interval.cast::<usize>().unwrap()))
        {
            if sigcnts[pos] >= self.sensitivity || cntcnts[pos] >= self.sensitivity {
                let difference = (sigcnts[pos] - cntcnts[pos])
                    .to_f64()
                    .ok_or_eyre("Failed to convert the NMS signal to f64")?;
                total += difference;
                minimum = minimum.min(difference);
                covered += 1;
            }
        }

        if covered == 0 {
            return Ok(None);
        }

        let baseline = Cnts::from(total / covered as f64 - minimum)
            .ok_or_eyre("Failed to convert the NMS baseline to the count type")?;
        let minimum = Cnts::from(minimum)
            .ok_or_eyre("Failed to convert the minimum NMS signal to the count type")?;
        if !baseline.is_finite() {
            return Err(eyre!("NMS baseline is not finite"));
        }
        if baseline <= self.sensitivity {
            return Ok(None);
        }

        let cutoff = baseline * self.fecutoff;
        if !cutoff.is_finite() {
            return Err(eyre!("NMS cutoff is not finite"));
        }
        Ok(Some((cutoff, minimum)))
    }

    fn filter_intervals(
        cutoff: Cnts,
        minimum: Cnts,
        intervals: &[Interval<Idx>],
        sigcnts: &[Cnts],
        cntcnts: &[Cnts],
        saveto: &mut Vec<Peak<Idx, Cnts>>,
    ) {
        let caller = ByCutoff {
            min_length: Idx::one(),
            merge_within: Idx::zero(),
            cutoff,
        };

        for interval in intervals {
            let iter = Range::from(interval.cast::<usize>().unwrap()).map(|pos| {
                // The round trip is safe: every position came from an Idx interval.
                let start = Idx::from(pos).unwrap();
                let end = Idx::from(pos + 1).unwrap();
                (start, end, (sigcnts[pos] - cntcnts[pos]) - minimum)
            });
            caller.run_from_iter(iter, saveto);
        }
    }

    fn nmsit_uniform(
        &self,
        nms: &[Interval<Idx>],
        sigcnts: &[Cnts],
        cntcnts: &[Cnts],
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

        let Some((cutoff, minimum)) = self.threshold(nms, sigcnts, cntcnts)? else {
            return Ok(());
        };

        Self::filter_intervals(cutoff, minimum, &overlapping, sigcnts, cntcnts, saveto);
        Ok(())
    }

    fn nmsit_slop(
        &self,
        nms: &[Interval<Idx>],
        sigcnts: &[Cnts],
        cntcnts: &[Cnts],
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

            let threshold = self.threshold(chain.links(), sigcnts, cntcnts)?;
            buffer = chain;
            let Some((cutoff, minimum)) = threshold else {
                continue;
            };

            for peak in group {
                let chain = match mapping.invmap_interval(peak, std::mem::take(&mut buffer)) {
                    Mapping::Complete(chain) | Mapping::Truncated(chain) => chain,
                    Mapping::None => return Err(eyre!("Failed to map an NMS peak region")),
                };
                Self::filter_intervals(cutoff, minimum, chain.links(), sigcnts, cntcnts, saveto);
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

    pub fn run(
        &self,
        orientation: Orientation,
        region: (Idx, Idx),
        peaks: &[Peak<Idx, Cnts>],
        sigcnts: &[Cnts],
        cntcnts: &[Cnts],
    ) -> Result<Vec<Peak<Idx, Cnts>>> {
        if peaks.is_empty() || self.roi[orientation].is_empty() {
            return Ok(Vec::new());
        }

        let region = Interval::new(region.0, region.1)?;
        let length = region
            .len()
            .to_usize()
            .ok_or_eyre("NMS region length exceeds usize")?;
        if sigcnts.len() != length || cntcnts.len() != length {
            return Err(eyre!(
                "NMS buffers must match the query length ({length}), got signal={} and control={}",
                sigcnts.len(),
                cntcnts.len()
            ));
        }
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
                    self.nmsit_uniform(&nms, sigcnts, cntcnts, &peaks[start..], &mut allowed)?;
                } else {
                    self.nmsit_slop(&nms, sigcnts, cntcnts, &peaks[start..], &mut allowed)?;
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

    fn chain(start: usize, end: usize) -> Result<ChainInterval<usize>> {
        ChainInterval::try_from_iter([Interval::new(start, end)?].into_iter())
    }

    fn peak(start: usize, end: usize, signal: f64, summit: usize) -> Peak<usize, f64> {
        Peak::new(start, end, signal, summit).unwrap()
    }

    #[test]
    fn unconfigured_nms_has_no_filtered_peaks() -> Result<()> {
        let nms = NMS::<usize, f64>::new();
        let peaks = vec![peak(1, 3, 4.0, 1)];
        let result = nms.run(
            Orientation::Forward,
            (100, 105),
            &peaks,
            &[0.0; 5],
            &[0.0; 5],
        )?;
        assert!(result.is_empty());
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
            &signal,
            &control,
        )?;

        assert_eq!(result.len(), 1);
        assert_eq!(*result[0].interval(), Interval::new(2, 4)?);
        assert_eq!(*result[0].signal(), 5.0);
        assert_eq!(*result[0].summit(), 2);
        Ok(())
    }

    #[test]
    fn empty_background_does_not_create_nan_peaks() -> Result<()> {
        let mut nms = NMS::<usize, f64>::new();
        nms.add_regions(
            Orientation::Forward,
            NMSRegions::new(vec![chain(0, 4)?], true)?,
        );

        let result = nms.run(
            Orientation::Forward,
            (0, 4),
            &[peak(0, 4, 4.0, 1)],
            &[0.0; 4],
            &[0.0; 4],
        )?;
        assert!(result.is_empty());
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
            &signal,
            &control,
        )?;

        assert_eq!(result.len(), 1);
        assert_eq!(*result[0].interval(), Interval::new(4, 6)?);
        Ok(())
    }

    #[test]
    fn local_contrast_must_exceed_sensitivity() -> Result<()> {
        let mut nms = NMS::<usize, f64>::new();
        nms.set_fecutoff(2.0)?.set_sensitivity(1.0)?;
        let region = [Interval::new(0, 2)?];
        let control = [1.0, 1.0];

        assert_eq!(nms.threshold(&region, &[1.0, 2.0], &control)?, None);
        assert_eq!(nms.threshold(&region, &[1.0, 3.0], &control)?, None);
        assert_eq!(
            nms.threshold(&region, &[1.0, 5.0], &control)?,
            Some((4.0, 0.0))
        );
        assert_eq!(nms.threshold(&region, &[1.0, 1.0], &control)?, None);
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
            &signal,
            &control,
        )?;

        assert_eq!(result.len(), 1);
        assert_eq!(*result[0].interval(), Interval::new(10, 11)?);
        Ok(())
    }

    #[test]
    fn configured_orientations_define_eligibility() -> Result<()> {
        let mut nms = NMS::<usize, f64>::new();
        nms.add_regions(
            Orientation::Forward,
            NMSRegions::new(vec![chain(0, 2)?], true)?,
        );

        let result = nms.run(
            Orientation::Reverse,
            (0, 2),
            &[peak(0, 2, 2.0, 0)],
            &[1.0; 2],
            &[1.0; 2],
        )?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_invalid_numerical_configuration() {
        let mut nms = NMS::<i64, f64>::new();
        assert!(nms.set_fecutoff(f64::NAN).is_err());
        assert!(nms.set_fecutoff(f64::INFINITY).is_err());
        assert!(nms.set_slopfrac(f32::NAN).is_err());
        assert!(nms.set_slopfrac(f32::INFINITY).is_err());
        assert!(nms.set_sensitivity(0.0).is_err());
        assert!(nms.set_sensitivity(f64::NAN).is_err());
        assert!(nms.set_sloplim(-1, 1).is_err());
        assert!(nms.set_sloplim(2, 1).is_err());
    }
}
