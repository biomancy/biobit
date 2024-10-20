use super::Resolution;
pub use crate::result::ResolutionOutcome;
use biobit_collections_rs::interval_tree::overlap;
use biobit_core_rs::num::{Num, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use derive_more::Constructor;

#[derive(Clone, Copy, Debug, Default, Constructor)]
pub struct Binary;

impl<Idx: PrimInt, Cnts: Num, Elt> Resolution<Idx, Cnts, Elt> for Binary {
    fn resolve(
        &mut self,
        _alignment: &SegmentedAlignment<Idx>,
        overlap: &mut overlap::Elements<Idx, usize>,
        _elements: &[Elt],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcome,
    ) {
        let mut empty = 0;
        for elements in overlap.annotations() {
            for ind in elements {
                counts[*ind] = counts[*ind] + Cnts::one();
            }
            empty += elements.is_empty() as u64;
        }

        outcome.discarded += empty;
        outcome.resolved += (overlap.len() as u64 - empty);
    }
}
