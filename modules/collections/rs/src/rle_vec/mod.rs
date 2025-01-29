pub use identical::Identical;
pub use merge::{merge, Merge, MergeFn};
pub use merge2::{merge2, Merge2, Merge2Fn};
pub use rle_vec::RleVec;

mod identical;
mod merge;
mod merge2;
#[allow(clippy::module_inception)]
mod rle_vec;
