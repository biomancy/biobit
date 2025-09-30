use std::ops::Range;

pub use crate::core::dna::NucCounts;
use biobit_core_rs::loc::PerOrientation;

pub mod cnt;
pub mod cntitem;

pub struct InnerNucCounts<'a, Data> {
    pub data: Data,
    pub range: Range<u64>,
    pub cnts: PerOrientation<Option<&'a [NucCounts]>>,
    pub coverage: PerOrientation<u32>,
}

pub struct NucCounterResult<'a, Data> {
    pub contig: &'a str,
    pub mapped: PerOrientation<u32>,
    pub cnts: Vec<InnerNucCounts<'a, Data>>,
}
