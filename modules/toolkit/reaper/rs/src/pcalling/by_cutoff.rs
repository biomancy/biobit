use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};

use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::num::{Float, PrimInt, PrimUInt};

use super::peak::Peak;

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, Getters)]
pub struct ByCutoff<Idx, Cnts> {
    pub min_length: Idx,
    pub merge_within: Idx,
    pub cutoff: Cnts,
}

impl<Idx: PrimInt, Cnts: Float> Default for ByCutoff<Idx, Cnts> {
    fn default() -> Self {
        Self {
            min_length: Idx::zero(),
            merge_within: Idx::zero(),
            cutoff: Cnts::zero(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float> ByCutoff<Idx, Cnts> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_min_length(&mut self, min_length: Idx) -> &mut Self {
        self.min_length = min_length;
        self
    }

    pub fn set_merge_within(&mut self, merge_within: Idx) -> &mut Self {
        self.merge_within = merge_within;
        self
    }

    pub fn set_cutoff(&mut self, cutoff: Cnts) -> &mut Self {
        self.cutoff = cutoff;
        self
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
            // Skip if below cutoff
            if val < self.cutoff {
                continue;
            }
            current = match current {
                None => Some((start, end, val, (start + end) / div)),
                Some(mut peak) => {
                    if start - peak.1 <= self.merge_within {
                        peak.1 = end;

                        // Update the summit if the signal is higher
                        if val > peak.2 {
                            peak.2 = val;
                            peak.3 = (start + end) / div;
                        }

                        Some(peak)
                    } else {
                        // Save the current peak if it is long enough
                        if peak.1 - peak.0 > self.min_length {
                            saveto.push(Peak::new(peak.0, peak.1, peak.2, peak.3).unwrap());
                        }
                        Some((start, end, val, (start + end) / div))
                    }
                }
            };
        }

        if let Some(peak) = current {
            if peak.1 - peak.0 > self.min_length {
                saveto.push(Peak::new(peak.0, peak.1, peak.2, peak.3).unwrap());
            }
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
