//! FASTQ asset formats and their run compatibility contracts.

pub mod paired;
pub mod single;

pub use paired::{PairedFastq, PairedFastqInput};
pub use single::{Fastq, FastqInput};

pub(crate) use paired::UnresolvedPairedFastq;
pub(crate) use single::UnresolvedFastq;
