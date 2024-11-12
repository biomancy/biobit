use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use biobit_collections_rs::interval_tree::overlap;
use biobit_collections_rs::interval_tree::overlap::Elements;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use itertools::{izip, Itertools};

use biobit_core_rs::loc::IntervalOp;

#[derive(Clone, Debug, Default)]
pub struct OverlapWeighted<Idx: PrimInt> {
    steps: overlap::Steps<Idx, usize>,
}

impl<Idx: PrimInt> OverlapWeighted<Idx> {
    pub fn new() -> Self {
        Self {
            steps: overlap::Steps::default(),
        }
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt> Resolution<Idx, Cnts, Elt> for OverlapWeighted<Idx> {
    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut [Elements<Idx, usize>],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) {
        for (query, n, overlap) in izip!(
            alignment.intervals.iter(),
            alignment.total_hits.iter(),
            overlap.iter()
        ) {
            debug_assert_eq!(query.len(), overlap.len());
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

            debug_assert!(
                self.steps
                    .iter()
                    .map(|x| x
                        .map(|(start, end, _)| end - start)
                        .fold(Idx::zero(), |sum, x| sum + x))
                    .fold(Idx::zero(), |sum, x| sum + x)
                    == length,
                "Query: {:?}\n{:?}\n{:?}",
                query.iter().collect_vec(),
                self.steps,
                overlap
            );
        }
    }
}
