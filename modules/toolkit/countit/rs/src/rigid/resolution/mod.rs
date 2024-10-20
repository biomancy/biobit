mod binary;
mod proportional;

use biobit_collections_rs::interval_tree::overlap;
use biobit_core_rs::num::{Num, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use dyn_clone::DynClone;
use impl_tools::autoimpl;

use crate::result::ResolutionOutcome;
pub use binary::Binary;

#[autoimpl(for <M: trait> Box<M> where Box<M>: Clone)]
pub trait Resolution<Idx: PrimInt, Cnts: Num, Elt>: DynClone + Send + Sync {
    fn reset(&mut self, _elements: &[Elt]) {}

    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut overlap::Elements<Idx, usize>,
        elements: &[Elt],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcome,
    );
}
