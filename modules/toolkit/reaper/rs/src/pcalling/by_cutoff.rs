#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};
use eyre::{Result, eyre};

use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::num::{Float, PrimInt, PrimUInt};

use super::peak::Peak;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, Getters)]
pub struct ByCutoff<Idx, Cnts> {
    pub min_length: Idx,
    pub merge_within: Idx,
    pub cutoff: Cnts,
}

impl<Idx: PrimInt, Cnts: Float> Default for ByCutoff<Idx, Cnts> {
    fn default() -> Self {
        Self {
            min_length: Idx::one(),
            merge_within: Idx::zero(),
            cutoff: Cnts::one(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float> ByCutoff<Idx, Cnts> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_min_length(&mut self, min_length: Idx) -> Result<&mut Self> {
        if min_length <= Idx::zero() {
            return Err(eyre!("Minimum peak length must be greater than zero"));
        }
        self.min_length = min_length;
        Ok(self)
    }

    pub fn set_merge_within(&mut self, merge_within: Idx) -> Result<&mut Self> {
        if merge_within < Idx::zero() {
            return Err(eyre!("Merge distance must be non-negative"));
        }
        self.merge_within = merge_within;
        Ok(self)
    }

    pub fn set_cutoff(&mut self, cutoff: Cnts) -> Result<&mut Self> {
        if !cutoff.is_finite() || cutoff <= Cnts::zero() {
            return Err(eyre!(
                "Peak-calling cutoff must be finite and greater than zero"
            ));
        }
        self.cutoff = cutoff;
        Ok(self)
    }

    pub fn run_from_iter(
        &self,
        iterator: impl Iterator<Item = (Idx, Idx, Cnts)>,
        saveto: &mut Vec<Peak<Idx, Cnts>>,
    ) {
        let div = Idx::from(2).unwrap();

        // Single pass peak calling
        let mut current = None; // (start, end, signal, summit)
        for (start, end, val) in iterator {
            debug_assert!(start < end);
            // Skip if below cutoff
            if val < self.cutoff {
                continue;
            }
            let summit = start + (end - start) / div;
            current = match current {
                None => Some((start, end, val, summit)),
                Some(mut peak) => {
                    debug_assert!(start >= peak.1);
                    if start - peak.1 <= self.merge_within {
                        peak.1 = end;

                        // Update the summit if the signal is higher
                        if val > peak.2 {
                            peak.2 = val;
                            peak.3 = summit;
                        }

                        Some(peak)
                    } else {
                        // Save the current peak if it is long enough
                        if peak.1 - peak.0 >= self.min_length {
                            saveto.push(Peak::new(peak.0, peak.1, peak.2, peak.3).unwrap());
                        }
                        Some((start, end, val, summit))
                    }
                }
            };
        }

        if let Some(peak) = current
            && peak.1 - peak.0 >= self.min_length
        {
            saveto.push(Peak::new(peak.0, peak.1, peak.2, peak.3).unwrap());
        }
    }

    pub fn run<L: PrimUInt, I: Identical<Cnts>>(
        &self,
        rle: &RleVec<Cnts, L, I>,
    ) -> Vec<Peak<Idx, Cnts>> {
        let mut cursor = Idx::zero();
        let iterator = rle.runs().map(|(val, length)| {
            let start = cursor;
            let end = start + Idx::from(*length).unwrap();
            cursor = end;
            (start, end, *val)
        });

        let mut result = Vec::new();
        self.run_from_iter(iterator, &mut result);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_exclude_zero_background_and_keep_one_base_peaks() {
        let caller = ByCutoff::<usize, f64>::new();
        let mut peaks = Vec::new();
        caller.run_from_iter([(0, 1, 0.0), (1, 2, 1.0)].into_iter(), &mut peaks);

        assert_eq!(peaks.len(), 1);
        assert_eq!(peaks[0].interval(), &(1, 2));
        assert_eq!(*peaks[0].signal(), 1.0);
    }

    #[test]
    fn minimum_length_is_inclusive() -> Result<()> {
        let mut caller = ByCutoff::<usize, f64>::new();
        caller.set_cutoff(2.0)?.set_min_length(2)?;

        let mut peaks = Vec::new();
        caller.run_from_iter([(4, 6, 2.0)].into_iter(), &mut peaks);
        assert_eq!(peaks.len(), 1);
        assert_eq!(peaks[0].interval(), &(4, 6));
        Ok(())
    }

    #[test]
    fn midpoint_does_not_overflow() {
        let caller = ByCutoff::<u64, f64>::new();
        let start = u64::MAX - 4;
        let end = u64::MAX - 2;
        let mut peaks = Vec::new();
        caller.run_from_iter([(start, end, 1.0)].into_iter(), &mut peaks);
        assert_eq!(*peaks[0].summit(), u64::MAX - 3);
    }

    #[test]
    fn rejects_invalid_configuration() {
        let mut caller = ByCutoff::<i64, f64>::new();
        assert!(caller.set_min_length(0).is_err());
        assert!(caller.set_merge_within(-1).is_err());
        assert!(caller.set_cutoff(0.0).is_err());
        assert!(caller.set_cutoff(f64::NAN).is_err());
        assert!(caller.set_cutoff(f64::INFINITY).is_err());
    }
}
