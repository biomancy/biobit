pub use identical::Identical;
pub use merge::{Merge, MergeFn, merge};
pub use merge2::{Merge2, Merge2Fn, merge2};
pub use rle_vec::RleVec;

mod identical;
mod merge;
mod merge2;
#[allow(clippy::module_inception)]
mod rle_vec;
