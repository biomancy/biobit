//! Dataset layouts composed from single-file FASTQ assets.

mod paired;
mod single;

pub use paired::{Pair, Paired, PairedId, PairedInput};
pub use single::{Single, SingleId, SingleInput};
