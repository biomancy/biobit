use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use ahash::HashSet;
use biobit_collections_rs::interval_tree::overlap;
use biobit_core_rs::num::{Float, Num, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use derive_getters::Getters;

#[derive(Clone, Debug, Default, Getters)]
pub struct AnyOverlap {
    downscale_multimappers: bool,
    cache: HashSet<usize>,
}

impl AnyOverlap {
    pub fn new(downscale_multimappers: bool) -> Self {
        Self {
            downscale_multimappers,
            cache: HashSet::default(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt> Resolution<Idx, Cnts, Elt> for AnyOverlap {
    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut [overlap::Elements<Idx, usize>],
        _elements: &[Elt],
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
