use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use ahash::HashSet;
use biobit_collections_rs::interval_tree::overlap::Elements;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use derive_getters::Getters;

#[derive(Clone, Debug, Default, Getters)]
pub struct AnyOverlap {
    cache: HashSet<usize>,
}

impl AnyOverlap {
    pub fn new() -> Self {
        Self {
            cache: HashSet::default(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt> Resolution<Idx, Cnts, Elt> for AnyOverlap {
    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut [Elements<Idx, usize>],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) {
        debug_assert_eq!(alignment.len(), overlap.len());

        let mut empty = 0;
        for (query, n) in overlap.iter_mut().zip(&alignment.total_hits) {
            // Gather unique hits
            self.cache.clear();
            for hit in query.annotations() {
                for ind in hit {
                    self.cache.insert(*ind);
                }
            }

            // Calculate the weight
            let weight = Cnts::one() / Cnts::from(*n).unwrap();

            // Update the counts
            for ind in self.cache.iter() {
                counts[*ind] = counts[*ind] + weight;
            }
            empty += self.cache.is_empty() as u64;
        }

        outcome.discarded = outcome.discarded + Cnts::from(empty).unwrap();
        outcome.resolved = outcome.resolved + Cnts::from(overlap.len() as u64 - empty).unwrap();
    }
}
