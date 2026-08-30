pub use identical::Identical;
pub use merge::{Merge, MergeSetup, TryMerge, merge};
pub use rle_vec::RleVec;

mod identical;
pub mod merge;
#[allow(clippy::module_inception)]
mod rle_vec;
