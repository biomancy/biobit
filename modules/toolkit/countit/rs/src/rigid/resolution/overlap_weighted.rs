use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use biobit_collections_rs::interval_tree::overlap;
use biobit_collections_rs::interval_tree::overlap::Elements;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use itertools::izip;

use biobit_core_rs::loc::AsSegment;

#[derive(Clone, Debug, Default)]
pub struct OverlapWeighted<Idx: PrimInt> {
    downscale_multimappers: bool,
    steps: overlap::Steps<Idx, usize>,
}

impl<Idx: PrimInt> OverlapWeighted<Idx> {
    pub fn new(downscale_multimappers: bool) -> Self {
        Self {
            downscale_multimappers,
            steps: overlap::Steps::default(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt> Resolution<Idx, Cnts, Elt> for OverlapWeighted<Idx> {
    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut [Elements<Idx, usize>],
        _elements: &[Elt],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) {
        for (query, n, overlap) in izip!(
            alignment.segments.iter(),
            alignment.total_hits.iter(),
            overlap.iter()
        ) {
            self.steps.build(query.iter().zip(overlap.iter()));

            let length: Idx = query
                .iter()
                .map(|x| x.len())
                .fold(Idx::zero(), |sum, x| sum + x);

            let weight = Cnts::one() / (Cnts::from(length).unwrap() * Cnts::from(*n).unwrap());
            for segment_steps in self.steps.iter() {
                for (start, end, hits) in segment_steps {
                    let segweight = Cnts::from(end - start).unwrap() * weight;

                    // consumed = consumed + weight;
                    if hits.is_empty() {
                        outcome.discarded = outcome.discarded + segweight;
                    } else {
                        outcome.resolved = outcome.resolved + segweight;
                        let segweight = segweight / Cnts::from(hits.len()).unwrap();
                        for x in hits {
                            counts[*x] = counts[*x] + segweight;
                        }
                    }
                }
            }
        }
    }
}
