//! Closed, analysis-ready file boundaries for acquisitions.

pub mod fastq;
mod root;

pub use root::{Dataset, DatasetId, DatasetIdRef, DatasetKind};
