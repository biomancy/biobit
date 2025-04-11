pub use bits::{Bits, BitsBuilder};
pub use results::{BatchHits, HitSegments, Hits, BatchHitSegments};
pub use tree::{Builder, ITree};

mod bits;
mod results;
pub mod tree;

#[cfg(test)]
pub(crate) mod test_stand;

// If I ever want to use a bundle for the IntervalTree, it should be called Forest.
// Re-working the module will require:
// * Support for caching allocations (required for hits/segments construction and trees)
// * Clearly defined primitives for intervals/pairs/chains/locations/etc
// * Guarantees about non-overlapping and sorted nature of input intervals.
