use derive_getters::{Dissolve, Getters};
use eyre::Result;

use biobit_core_rs::loc::AsSegment;
use biobit_core_rs::num::{Float, PrimInt};

use crate::pcalling::ByCutoff;
use crate::result::Peak;

#[derive(Clone, PartialEq, PartialOrd, Debug, Dissolve, Getters)]
pub struct NMS<Idx, Cnts> {
    fecutoff: Cnts,
    group_within: Idx,
    slopfrac: f32,
    sloplim: (Idx, Idx),
    // Boundaries are in the region coordinates
    boundaries: Vec<Idx>,
}

impl<Idx: PrimInt, Cnts: Float> NMS<Idx, Cnts> {
    pub fn new() -> Self {
        NMS {
            fecutoff: Cnts::one(),
            group_within: Idx::zero(),
            slopfrac: 1.0,
            sloplim: (Idx::min_value(), Idx::max_value()),
            boundaries: Vec::new(),
        }
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

    pub fn set_boundaries(&mut self, mut boundaries: Vec<Idx>) -> &mut Self {
        boundaries.sort();
        boundaries.dedup();

        self.boundaries = boundaries;
        self
    }

    pub fn set_boundaries_trusted(&mut self, boundaries: Vec<Idx>) -> &mut Self {
        self.boundaries = boundaries;
        self
    }

    pub fn run(
        &self,
        mut peaks: Vec<Peak<Idx, Cnts>>,
        signal: &[Cnts],
    ) -> Result<Vec<Peak<Idx, Cnts>>> {
        if peaks.is_empty() {
            return Ok(Vec::new());
        }
        peaks.sort_by_key(|x| x.segment().start());

        // Group peaks within X bases
        let mut groups = vec![];

        let mut drain = peaks.drain(..);
        let mut last = drain.next().unwrap();
        let mut cache = vec![];

        for p in drain {
            debug_assert!(p.segment().start() > last.segment().end());

            if p.segment().start() - last.segment().end() > self.group_within {
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

        debug_assert!(peaks.is_empty());
        for group in groups.into_iter() {
            // Group limits
            let (start, end) = (
                group.first().unwrap().segment().start(),
                group.last().unwrap().segment().end(),
            );

            // Slop size
            let slop = Idx::from((end - start).to_f32().unwrap() * self.slopfrac)
                .and_then(|x| Some(x.clamp(self.sloplim.0, self.sloplim.1)))
                .unwrap_or(self.sloplim.1);

            // Coordinates of the slopped region
            let slop_start = left_slop(&self.boundaries, start, slop).to_usize().unwrap();
            let slop_end = right_slop(&self.boundaries, end, slop)
                .min(end)
                .to_usize()
                .unwrap();
            if slop_start >= slop_end {
                continue;
            }

            // Calculate the mean signal in the slopped region
            let total: f64 = signal[slop_start..slop_end]
                .iter()
                .map(|x| x.to_f64().unwrap())
                .sum();
            let baseline = total / (slop_end - slop_start) as f64 + 1e-6;

            // Subdivided the original peak into smaller sub-peaks passing the NMS filter
            let bycutoff = ByCutoff {
                min_length: Idx::one(),
                merge_within: Idx::zero(),
                cutoff: Cnts::from(baseline).unwrap(),
            };

            for peak in group {
                let start = peak.segment().start().to_usize().unwrap();
                let end = peak.segment().end().to_usize().unwrap();

                let iterator = signal[start..end].iter().enumerate().map(|(ind, x)| {
                    (
                        Idx::from(start + ind).unwrap(),
                        Idx::from(start + ind + 1).unwrap(),
                        x.clone(),
                    )
                });

                bycutoff.run_from_iter(iterator, &mut peaks);
            }
        }

        Ok(peaks)
    }
}

fn left_slop<Idx: PrimInt>(boundaries: &[Idx], pos: Idx, maxdist: Idx) -> Idx {
    match boundaries.binary_search(&pos) {
        // Pos exists in the boundaries index, can't move to the left
        Ok(_) => pos,
        // Pos is to the left of the first boundary, slop up to 0
        Err(0) => pos - maxdist.min(pos),
        // Pos is somewhere between boundaries, slop towards i-1-th boundary
        Err(ind) => {
            let slopped = pos - maxdist.min(pos);
            boundaries[ind - 1].max(slopped)
        }
    }
}

fn right_slop<Idx: PrimInt>(boundaries: &[Idx], pos: Idx, maxdist: Idx) -> Idx {
    match boundaries.binary_search(&pos) {
        // Pos exists in the boundaries index, can't move to the right
        Ok(_) => pos,
        // Pos is to the right of the last boundary, slop freely
        Err(ind) if ind == boundaries.len() => pos + maxdist,
        // Pos is somewhere between boundaries, slop towards i-th boundary
        Err(ind) => boundaries[ind].min(pos + maxdist),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left_slop() {
        let arr: Vec<u8> = vec![10, 20, 30, 40];
        // Before the first boundary
        assert_eq!(left_slop(&arr, 0, 5), 0);
        assert_eq!(left_slop(&arr, 5, 10), 0);
        assert_eq!(left_slop(&arr, 8, 5), 3);
        // On the boundary
        assert_eq!(left_slop(&arr, 10, 5), 10);
        assert_eq!(left_slop(&arr, 20, 100), 20);
        assert_eq!(left_slop(&arr, 40, 3), 40);
        // Between boundaries
        assert_eq!(left_slop(&arr, 15, 10), 10);
        assert_eq!(left_slop(&arr, 25, 6), 20);
        assert_eq!(left_slop(&arr, 25, 3), 22);
        assert_eq!(left_slop(&arr, 25, 0), 25);
        // After the last boundary
        assert_eq!(left_slop(&arr, 50, 5), 45);
        assert_eq!(left_slop(&arr, 50, 100), 40);

        // Empty boundaries
        let arr: Vec<u8> = vec![];
        assert_eq!(left_slop(&arr, 0, 5), 0);
        assert_eq!(left_slop(&arr, 5, 10), 0);
        assert_eq!(left_slop(&arr, 10, 5), 5);
    }

    #[test]
    fn test_right_slop() {
        let arr: Vec<u8> = vec![10, 20, 30, 40];
        // Before the first boundary
        assert_eq!(right_slop(&arr, 0, 5), 5);
        assert_eq!(right_slop(&arr, 5, 10), 10);
        assert_eq!(right_slop(&arr, 8, 5), 10);
        // On the boundary
        assert_eq!(right_slop(&arr, 10, 5), 10);
        assert_eq!(right_slop(&arr, 20, 100), 20);
        assert_eq!(right_slop(&arr, 40, 3), 40);
        // Between boundaries
        assert_eq!(right_slop(&arr, 15, 10), 20);
        assert_eq!(right_slop(&arr, 25, 3), 28);
        assert_eq!(right_slop(&arr, 25, 5), 30);
        assert_eq!(right_slop(&arr, 25, 0), 25);
        // After the last boundary
        assert_eq!(right_slop(&arr, 50, 5), 55);
        assert_eq!(right_slop(&arr, 50, 100), 150);

        // Empty boundaries
        let arr: Vec<u8> = vec![];
        assert_eq!(right_slop(&arr, 0, 5), 5);
        assert_eq!(right_slop(&arr, 5, 10), 15);
        assert_eq!(right_slop(&arr, 10, 5), 15);
    }
}