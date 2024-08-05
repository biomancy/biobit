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
    // In global coordinates:
    // * Regions that passed modelling thresholds
    // * Raw peaks in global coordinates
    // * NMS peaks in global coordinates
    modeled: Vec<Segment<Idx>>,
    raw_peaks: Vec<Peak<Idx, Cnts>>,
    filtered_peaks: Vec<Peak<Idx, Cnts>>,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Harvest<Ctg: Contig, Idx: PrimInt, Cnts: Float, Tag> {
    comparison: Tag,
    regions: Vec<HarvestRegion<Ctg, Idx, Cnts>>,
}
