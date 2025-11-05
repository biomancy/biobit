use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use biobit_collections_rs::interval_tree;
use biobit_core_rs::loc::IntervalOp;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use eyre::Result;
use itertools::{Itertools, izip};

#[derive(Clone, Debug, Default)]
pub struct OverlapWeighted<Idx: PrimInt> {
    steps: Option<interval_tree::BatchHitSegments<'static, Idx, usize>>,
}

impl<Idx: PrimInt> OverlapWeighted<Idx> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt> Resolution<Idx, Cnts, Elt> for OverlapWeighted<Idx> {
    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        bhits: &mut interval_tree::BatchHits<Idx, usize>,
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) -> Result<()> {
        let mut steps = self.steps.take().unwrap_or_default().recycle();
        steps.build(alignment.intervals.iter(), bhits)?;

        for (query, n, (segments, data)) in izip!(
            alignment.intervals.iter(),
            alignment.total_hits.iter(),
            steps.iter(),
        ) {
            let length: Idx = query
                .iter()
                .map(|x| x.len())
                .fold(Idx::zero(), |sum, x| sum + x);
            let weight = Cnts::one() / (Cnts::from(length).unwrap() * Cnts::from(*n).unwrap());

            for (segment, hits) in izip!(segments, data) {
                let segweight = Cnts::from(segment.len()).unwrap() * weight;
                if hits.is_empty() {
                    outcome.discarded = outcome.discarded + segweight;
                } else {
                    outcome.resolved = outcome.resolved + segweight;
                    let segweight = segweight / Cnts::from(hits.len()).unwrap();
                    for x in hits {
                        counts[**x] = counts[**x] + segweight;
                    }
                }
            }

            debug_assert!(
                segments
                    .iter()
                    .map(|x| x.len())
                    .fold(Idx::zero(), |sum, x| sum + x)
                    == length,
                "Query: {:?}\n{:?}\n{:?}",
                query.iter().collect_vec(),
                segments,
                data
            );
        }

        // Recycle the steps
        self.steps = Some(steps.recycle());

        Ok(())
    }
}
