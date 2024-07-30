use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;

use biobit_core_rs::loc::{Contig, PerOrientation, Segment};
use biobit_core_rs::num::{Float, PrimInt};

pub use super::pcalling::Peak;

// #[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
// pub struct Stats {
//     // Time spent processing the partition
//     time_s: f64,
//     // Elements inside and outside the annotation
//     inside_annotation: f64,
//     outside_annotation: f64,
// }

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Region<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // Query locus
    contig: Ctg,
    segment: Segment<Idx>,
    // General statistics
    // stats: Stats,
    // Peaks
    peaks: PerOrientation<Vec<Peak<Idx, Cnts>>>,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Ripped<Ctg: Contig, Idx: PrimInt, Cnts: Float, Tag> {
    comparison: Tag,
    regions: Vec<Region<Ctg, Idx, Cnts>>,
}
