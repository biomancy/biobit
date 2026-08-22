//! Closed, analysis-ready file boundaries for acquisitions.

pub mod fastq;
mod id;

pub use id::{DatasetId, DatasetIdRef};

use crate::provenance::AcquisitionIdRef;
use crate::{Meta, NonEmpty};
use serde::{Deserialize, Serialize};

/// A complete stored form of one acquisition.
///
/// Different datasets normally preserve the same acquisition data using
/// different file layouts or encodings. Reusing assets across datasets is
/// allowed, including for explicitly selected QC subsets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Dataset {
    /// One or more independent FASTQ files.
    Fastq(fastq::Single),
    /// One or more ordered read-one/read-two FASTQ pairs.
    PairedFastq(fastq::Paired),
}

impl Dataset {
    /// Returns this dataset's concrete borrowed identifier.
    pub fn id(&self) -> DatasetIdRef<'_> {
        match self {
            Self::Fastq(dataset) => DatasetIdRef::Fastq(dataset.id()),
            Self::PairedFastq(dataset) => DatasetIdRef::PairedFastq(dataset.id()),
        }
    }

    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::Fastq(dataset) => dataset.acquisition().id(),
            Self::PairedFastq(dataset) => dataset.acquisition().id(),
        }
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        match self {
            Self::Fastq(dataset) => dataset.meta(),
            Self::PairedFastq(dataset) => dataset.meta(),
        }
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        match self {
            Self::Fastq(dataset) => dataset.description(),
            Self::PairedFastq(dataset) => dataset.description(),
        }
    }
}

impl From<fastq::Single> for Dataset {
    fn from(dataset: fastq::Single) -> Self {
        Self::Fastq(dataset)
    }
}

impl From<fastq::Paired> for Dataset {
    fn from(dataset: fastq::Paired) -> Self {
        Self::PairedFastq(dataset)
    }
}
