mod any_overlap;
mod overlap_weighted;

mod top_ranked;

use biobit_collections_rs::interval_tree::overlap;
use biobit_core_rs::num::{Float, Num, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use dyn_clone::DynClone;
use impl_tools::autoimpl;

use crate::result::ResolutionOutcomes;
pub use any_overlap::AnyOverlap;
pub use overlap_weighted::OverlapWeighted;
pub use top_ranked::TopRanked;

#[autoimpl(for <M: trait> Box<M> where Box<M>: Clone)]
pub trait Resolution<Idx: PrimInt, Cnts: Float, Elt>: DynClone + Send + Sync {
    fn reset(&mut self, _elements: &[Elt]) {}

    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut [overlap::Elements<Idx, usize>],
        elements: &[Elt],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    );
}
