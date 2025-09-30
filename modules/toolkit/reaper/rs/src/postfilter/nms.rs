use crate::pcalling::ByCutoff;
use crate::result::Peak;
use biobit_core_rs::loc::mapping::{ChainMap, Mapping};
use biobit_core_rs::loc::{ChainInterval, Interval, IntervalOp, Orientation, PerOrientation};
use biobit_core_rs::num::{Float, PrimInt};
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::Getters;
use derive_more::Into;
use eyre::Result;
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
            return Err(eyre::eyre!("NMS regions must not be empty"));
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
            sloplim: (Idx::min_value(), Idx::max_value()),
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
        if fecutoff < Cnts::one() {
            return Err(eyre::eyre!("NMS cutoff must be greater than 1"));
        }

        self.fecutoff = fecutoff;
        Ok(self)
    }

    pub fn set_group_within(&mut self, group_within: Idx) -> Result<&mut Self> {
        if group_within < Idx::zero() {
            return Err(eyre::eyre!("Group within must be greater than 0"));
        }

        self.group_within = group_within;
        Ok(self)
    }

    pub fn set_slopfrac(&mut self, slopfrac: f32) -> Result<&mut Self> {
        if slopfrac < 0.0 {
            return Err(eyre::eyre!("Slop fraction must be greater than 0"));
        }

        self.slopfrac = slopfrac;
        Ok(self)
    }

    pub fn set_sloplim(&mut self, minslop: Idx, maxslop: Idx) -> Result<&mut Self> {
        if minslop > maxslop {
            return Err(eyre::eyre!("Minimum slop must be less than maximum slop"));
        } else if minslop < Idx::zero() {
            return Err(eyre::eyre!("Minimum slop must be greater than 0"));
        }

        self.sloplim = (minslop, maxslop);
        Ok(self)
    }

    pub fn set_sensitivity(&mut self, sensitivity: Cnts) -> Result<&mut Self> {
        if sensitivity <= Cnts::zero() {
            return Err(eyre::eyre!("Sensitivity must be greater than 0"));
        }

        self.sensitivity = sensitivity;
        Ok(self)
    }

    pub fn add_regions(&mut self, orientation: Orientation, regions: NMSRegions<Idx>) -> &mut Self {
        self.roi[orientation].push(regions);
        self
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

        // Collect peaks that are within the NMS region
        let mut overlapping = Vec::with_capacity(peaks.len());
        for peak in peaks {
            if peak.interval().start() >= nms.last().unwrap().end() {
                break;
            } else if peak.interval().end() <= nms.first().unwrap().start() {
                continue;
            }

            for nms in nms.iter() {
                if nms.start() >= peak.interval().end() {
                    break;
                } else if nms.end() <= peak.interval().start() {
                    continue;
                }

                if let Some(x) = peak.interval().intersection(nms) {
                    overlapping.push(x);
                }
            }
        }
        if overlapping.is_empty() {
            return Ok(());
        }

        // Calculate the baseline for the whole NMS region
        let iter = nms
            .iter()
            .flat_map(|x| Range::from(x.cast::<usize>().unwrap()));

        let (mut baseline, mut minval, mut length) = (0.0f64, f64::INFINITY, 0usize);
        for pos in iter {
            if sigcnts[pos] >= self.sensitivity || cntcnts[pos] >= self.sensitivity {
                let value = (sigcnts[pos] - cntcnts[pos]).to_f64().unwrap();
                minval = minval.min(value);
                baseline += value;
                length += 1;
            }
        }

        let baseline = Cnts::from((baseline / length as f64) - minval).unwrap();
        let minval = Cnts::from(minval).unwrap();

        if baseline <= self.sensitivity {
            return Ok(());
        }

        // Subdivided the original peak into smaller sub-peaks passing the NMS filter
        let bycutoff = ByCutoff {
            min_length: Idx::one(),
            merge_within: Idx::zero(),
            cutoff: baseline * self.fecutoff,
        };

        for peak in overlapping {
            let iter = Range::from(peak.cast::<usize>().unwrap()).map(|x| {
                (
                    Idx::from(x).unwrap(),
                    Idx::from(x + 1).unwrap(),
                    (sigcnts[x] - cntcnts[x]) - minval,
                )
            });
            bycutoff.run_from_iter(iter, saveto);
        }

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

        // Create a mapping from the NMS region to the input peaks
        let mapping = ChainMap::new(ChainInterval::try_from_iter(nms.iter().cloned()).unwrap());

        // Map input peaks to the NMS region
        let mut mapped = Vec::new();
        for peak in peaks {
            if peak.interval().start() >= nms.last().unwrap().end() {
                break;
            } else if peak.interval().end() <= nms.first().unwrap().start() {
                continue;
            }
            match mapping.map_interval(peak.interval()) {
                Mapping::Complete(x) | Mapping::Truncated(x) => mapped.push(x),
                Mapping::None => continue,
            }
        }
        if mapped.is_empty() {
            return Ok(());
        }

        // Group peaks within X bases
        let groups = group_within(&mapped, self.group_within);

        // NMS each region
        let mut buffer = ChainInterval::default();
        for group in groups {
            // Group limits
            let (start, end) = (group.first().unwrap().start(), group.last().unwrap().end());

            // Slop size
            let slop = Idx::from((end - start).to_f32().unwrap() * self.slopfrac).unwrap();
            let slop = slop.clamp(self.sloplim.0, self.sloplim.1);

            // Backmap the slopped region to the global coordinates
            let query = Interval::new(start.saturating_sub(slop), end + slop)?;
            let chain = match mapping.invmap_interval(&query, std::mem::take(&mut buffer)) {
                Mapping::Complete(x) | Mapping::Truncated(x) => x,
                Mapping::None => {
                    debug_assert!(false, "Region must be mapped");
                    continue;
                }
            };

            // Calculate the baseline
            let iter = chain
                .links()
                .iter()
                .flat_map(|x| Range::from(x.cast::<usize>().unwrap()));

            let (mut baseline, mut minval) = (0.0f64, f64::INFINITY);
            for pos in iter {
                if sigcnts[pos] >= self.sensitivity || cntcnts[pos] >= self.sensitivity {
                    let value = (sigcnts[pos] - cntcnts[pos]).to_f64().unwrap();
                    minval = minval.min(value);
                    baseline += value;
                }
            }
            buffer = chain;

            let length = (end - start).to_f64().unwrap();
            let baseline = Cnts::from((baseline / length) - minval).unwrap();
            let minval = Cnts::from(minval).unwrap();

            if baseline <= self.sensitivity {
                continue;
            }

            // Subdivided the original peak into smaller sub-peaks passing the NMS filter
            let bycutoff = ByCutoff {
                min_length: Idx::one(),
                merge_within: Idx::zero(),
                cutoff: baseline * self.fecutoff,
            };

            for peak in group {
                // Map the peak to the global coordinates
                let chain = match mapping.invmap_interval(peak, std::mem::take(&mut buffer)) {
                    Mapping::Complete(x) | Mapping::Truncated(x) => x,
                    Mapping::None => {
                        debug_assert!(false, "Peak must be mapped");
                        continue;
                    }
                };

                let iter = chain
                    .links()
                    .iter()
                    .flat_map(|x| Range::from(x.cast::<usize>().unwrap()))
                    .map(|ind| {
                        (
                            Idx::from(ind).unwrap(),
                            Idx::from(ind + 1).unwrap(),
                            (sigcnts[ind] - cntcnts[ind]) - minval,
                        )
                    });
                bycutoff.run_from_iter(iter, saveto);
                buffer = chain;
            }
        }
        Ok(())
    }

    pub fn run(
        &self,
        orientation: Orientation,
        region: (Idx, Idx),
        peaks: &[Peak<Idx, Cnts>],
        sigcnts: &[Cnts],
        cntcnts: &[Cnts],
    ) -> Result<Vec<Peak<Idx, Cnts>>> {
        if peaks.is_empty() {
            return Ok(Vec::new());
        }
        debug_assert!(peaks.is_sorted_by_key(|x| x.interval().start()));

        let region = Interval::new(region.0, region.1)?;
        debug_assert!(region.len().to_usize().unwrap() == sigcnts.len());
        debug_assert!(region.len().to_usize().unwrap() == cntcnts.len());

        // Apply NMS filters
        let mut allowed = Vec::with_capacity(peaks.len());

        for config in &self.roi[orientation] {
            for nms in &config.regions {
                // Map the NMS region to the region coordinates
                let nms = nms
                    .links()
                    .iter()
                    .filter_map(|x| x.clamped(&region))
                    .map(|x| x << region.start())
                    .collect::<Vec<_>>();
                if nms.is_empty() {
                    continue;
                }

                // Find the peaks slice that overlaps with the region
                let start =
                    match peaks.binary_search_by_key(&nms[0].start(), |x| x.interval().start()) {
                        Ok(x) => x,
                        Err(0) => 0,
                        Err(x) => x - 1,
                    };

                // Apply the filter
                if config.uniform_baseline {
                    self.nmsit_uniform(&nms, sigcnts, cntcnts, &peaks[start..], &mut allowed)?
                } else {
                    self.nmsit_slop(&nms, sigcnts, cntcnts, &peaks[start..], &mut allowed)?
                }
            }
        }

        Ok(allowed)
    }
}

fn group_within<Idx: PrimInt>(
    intervals: &[Interval<Idx>],
    group_within: Idx,
) -> Vec<Vec<&Interval<Idx>>> {
    // Group peaks within X bases
    let mut groups = vec![];

    let mut peaks_iter = intervals.iter();
    let mut last = peaks_iter.next().unwrap();
    let mut cache = vec![];

    for p in peaks_iter {
        debug_assert!(p.start() >= last.end());

        if p.start() - last.end() > group_within {
            cache.push(last);
            groups.push(cache);

            last = p;
            cache = vec![];
        } else {
            cache.push(last);
            last = p;
        }
    }
    cache.push(last);
    groups.push(cache);

    groups
}
