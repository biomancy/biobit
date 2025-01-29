#![allow(clippy::too_many_arguments)]
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;

use biobit_core_rs::loc::{Contig, Interval, Orientation};
use biobit_core_rs::num::{Float, PrimInt};

pub use crate::pcalling::Peak;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct HarvestRegion<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // Region coordinates
    contig: Ctg,
    orientation: Orientation,
    interval: Interval<Idx>,
    // In global coordinates:
    // * Regions that were covered by at least 1 read in signal/control experiments
    // * Regions that passed modelling thresholds
    // * Raw peaks
    // * NMS peaks
    signal: Vec<Interval<Idx>>,
    control: Vec<Interval<Idx>>,
    modeled: Vec<Interval<Idx>>,
    raw_peaks: Vec<Peak<Idx, Cnts>>,
    filtered_peaks: Vec<Peak<Idx, Cnts>>,
}

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Harvest<Ctg: Contig, Idx: PrimInt, Cnts: Float, Tag> {
    comparison: Tag,
    regions: Vec<HarvestRegion<Ctg, Idx, Cnts>>,
}
