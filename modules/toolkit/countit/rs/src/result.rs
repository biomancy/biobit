use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;

use biobit_core_rs::loc::{Contig, Segment};
use biobit_core_rs::num::{Float, PrimInt};

#[derive(Clone, PartialEq, Debug, Default, Dissolve)]
pub struct ResolutionOutcome {
    pub resolved: u64,
    pub discarded: u64,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve)]
pub struct Summary<Ctg: Contig, Idx: PrimInt> {
    // Location of the processed genomic segment
    pub contig: Ctg,
    pub segment: Segment<Idx>,
    // Time spent processing the partition
    pub time_s: f64,
    // Number of alignments resolved and counted or discarded
    pub alignments: ResolutionOutcome,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Counts<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt, SrcTag> {
    pub source: SrcTag,
    pub elements: Vec<Elt>,
    pub counts: Vec<Cnts>,
    pub summaries: Vec<Summary<Ctg, Idx>>,
}
