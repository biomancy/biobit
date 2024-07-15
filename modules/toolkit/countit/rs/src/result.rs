use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;

use biobit_core_rs::loc::{Contig, Segment};
use biobit_core_rs::num::{Float, PrimInt};

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Stats<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // Location of the processed genomic segment
    contig: Ctg,
    segment: Segment<Idx>,
    // Time spent processing the partition
    time_s: f64,
    // Elements inside and outside the annotation
    inside_annotation: Cnts,
    outside_annotation: Cnts,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Counts<Ctg: Contig, Idx: PrimInt, Cnts: Float, Data, Tag> {
    source: Tag,
    data: Vec<Data>,
    counts: Vec<Cnts>,
    stats: Vec<Stats<Ctg, Idx, Cnts>>,
}
