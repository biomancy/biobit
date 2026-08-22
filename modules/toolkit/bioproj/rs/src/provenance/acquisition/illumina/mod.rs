//! Illumina sequencing acquisitions.

mod paired_end;
mod single_end;
mod validate;

pub use paired_end::{PairedEndSequencing, PairedEndSequencingId};
pub use single_end::{SingleEndSequencing, SingleEndSequencingId};
