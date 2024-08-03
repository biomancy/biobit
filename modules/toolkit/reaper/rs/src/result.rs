use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;

use biobit_core_rs::loc::{Contig, Orientation, Segment};
use biobit_core_rs::num::{Float, PrimInt};

pub use crate::pcalling::Peak;

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct HarvestRegion<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // Region coordinates
    contig: Ctg,
    orientation: Orientation,
    segment: Segment<Idx>,
    // Ripped peaks in global coordinates
    peaks: Vec<Peak<Idx, Cnts>>,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Harvest<Ctg: Contig, Idx: PrimInt, Cnts: Float, Tag> {
    comparison: Tag,
    regions: Vec<HarvestRegion<Ctg, Idx, Cnts>>,
}
