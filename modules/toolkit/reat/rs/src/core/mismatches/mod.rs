use biobit_core_rs::loc::{Orientation, PerOrientation};

pub mod prefilters;
pub mod site;

pub type StrandingCounts = PerOrientation<usize>;

pub trait MismatchesVec: Sized {
    fn contig(&self) -> &str;
    fn trstrand(&self) -> Orientation;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub trait Builder<'a> {
    type Out: MismatchesVec;
    type SourceCounts;
    fn build(&mut self, nc: Self::SourceCounts) -> Batch<Self::Out>;
}

pub struct Batch<T: MismatchesVec> {
    pub contig: String,
    pub mapped: PerOrientation<u32>,
    // Must be retained & printed no matter what
    pub retained: PerOrientation<T>,
    // Other mismatches
    pub items: PerOrientation<T>,
}
