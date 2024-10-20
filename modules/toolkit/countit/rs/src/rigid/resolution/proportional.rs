use super::Resolution;
pub use crate::result::ResolutionOutcome;
use biobit_collections_rs::interval_tree::overlap;
use biobit_collections_rs::interval_tree::overlap::Elements;
use biobit_core_rs::num::{Num, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;

#[derive(Clone, Debug, Default)]
pub struct Proportional<Idx: PrimInt> {
    steps: overlap::Steps<Idx, usize>,
}

impl<Idx: PrimInt, Cnts: Num, Elt> Resolution<Idx, Cnts, Elt> for Proportional<Idx> {
    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut Elements<Idx, usize>,
        elements: &[Elt],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcome,
    ) {
        // There is a clean problem with Elements here.
        // Each query element might be multisegmented. And I need to keep track of it (somehow).
        // I.e. I need to have a 2 level clear mapping. Batch -> query -> hits.
        // Damn, that's hard stuff.

        // // The hard stuff is happening here. The actual counting is done here.
        // steps.build(segments.iter().zip(overlaps.iter()));
        //
        // let length: Idx = segments
        //     .iter()
        //     .map(|x| x.len())
        //     .fold(Idx::zero(), |sum, x| sum + x);
        //
        // let weight =
        //     Cnts::one() / (Cnts::from(length).unwrap() * Cnts::from(n).unwrap());
        //
        // for segment_steps in steps.iter() {
        //     for (start, end, hits) in segment_steps {
        //         let segweight = Cnts::from(end - start).unwrap() * weight;
        //
        //         // consumed = consumed + weight;
        //         if hits.is_empty() {
        //             outside_annotation = outside_annotation + segweight;
        //         } else {
        //             inside_annotation = inside_annotation + segweight;
        //             let segweight = segweight / Cnts::from(hits.len()).unwrap();
        //             for x in hits {
        //                 cache.cnts[*x] = cache.cnts[*x] + segweight;
        //             }
        //         }
        //     }
        // }



        todo!()
    }
}
