use derive_more::{From, Into};

use biobit_core_rs::loc::{Contig, Interval};
use biobit_core_rs::num::{Float, PrimInt};

#[derive(Clone, PartialEq, PartialOrd, Debug, Default, From, Into)]
pub struct ResolutionOutcomes<Cnts: Float> {
    pub resolved: Cnts,
    pub discarded: Cnts,
}

#[derive(Clone, PartialEq, Debug, Default, From, Into)]
pub struct PartitionMetrics<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // Location of the processed partition
    pub contig: Ctg,
    pub interval: Interval<Idx>,
    // Time spent processing the partition
    pub time_s: f64,
    // Number of objects (e.g., alignments) that were resolved and counted or discarded
    pub outcomes: ResolutionOutcomes<Cnts>,
}

#[derive(Clone, PartialEq, Debug, Default, From, Into)]
pub struct Counts<'a, Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt, SrcTag> {
    pub source: SrcTag,
    pub elements: &'a [Elt],
    pub counts: Vec<Cnts>,
    pub partitions: Vec<PartitionMetrics<Ctg, Idx, Cnts>>,
}
