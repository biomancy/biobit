pub use mismatches::ByMismatches;

mod intervals;
mod mismatches;
use std::ops::Range;

pub use intervals::RetainSitesFromIntervals;

pub trait MismatchesPreFilter<T> {
    fn is_ok(&self, preview: &T) -> bool;
}

pub trait SitesRetainer {
    fn retained(&self, contig: &str, range: Range<u64>) -> Vec<Range<u64>>;
}
