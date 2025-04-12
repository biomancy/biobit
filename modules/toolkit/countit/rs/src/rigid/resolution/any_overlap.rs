use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use ahash::HashSet;
use biobit_collections_rs::interval_tree::BatchHits;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use derive_getters::Getters;
use eyre::Result;

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
        bhits: &mut BatchHits<'_, Idx, usize>,
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) -> Result<()> {
        debug_assert_eq!(alignment.len(), bhits.len());

        let mut empty = 0;
        for (query, n) in bhits.iter().zip(&alignment.total_hits) {
            // Gather unique hits
            self.cache.clear();
            for hit in query.1 {
                self.cache.insert(**hit);
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
        outcome.resolved = outcome.resolved + Cnts::from(bhits.len() as u64 - empty).unwrap();

        Ok(())
    }
}
