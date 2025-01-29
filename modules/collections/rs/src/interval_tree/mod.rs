pub use bits::{Bits, BitsBuilder};
pub use traits::{Builder, ITree, Record};

mod bits;
pub mod overlap;
pub mod traits;

// If I want to use a bundle for the IntervalTree, it should be called Forest.
