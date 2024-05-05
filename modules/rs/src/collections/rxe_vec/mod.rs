/// This module contains implementation for Vectors of Runs, encoded via either their length
/// (`RleVec`), or by their end position (`RpeVec`). Access to the data is provided through
/// `view::Flat`, `view::Runs`, and `view::EndPos` views.

pub use rle_vec::RleVec;
pub use rpe_vec::RpeVec;
pub use traits::{Identical, Length, Position};

mod traits;
mod rle_vec;
mod rpe_vec;
pub mod view;
