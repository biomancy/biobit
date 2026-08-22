//! Logical data acquisitions grouped by measurement family.

mod id;
pub mod illumina;

pub use id::{AcquisitionId, AcquisitionIdRef};

use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::Library;

/// A concrete logical acquisition.
///
/// This initial type intentionally combines two concepts: the reusable assay
/// specification and one execution of that assay. A future model may separate
/// them so repeated acquisitions can reference one shared method definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Acquisition {
    /// Single-end Illumina sequencing.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencing),
    /// Paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencing),
}

impl Acquisition {
    /// Returns this acquisition's concrete borrowed identifier.
    pub fn id(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => {
                AcquisitionIdRef::IlluminaSingleEndSequencing(acquisition.id())
            }
            Self::IlluminaPairedEndSequencing(acquisition) => {
                AcquisitionIdRef::IlluminaPairedEndSequencing(acquisition.id())
            }
        }
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.meta(),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.meta(),
        }
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.description(),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.description(),
        }
    }

    /// Validates this acquisition's material compatibility contract.
    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.validate(libraries),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.validate(libraries),
        }
    }
}

impl From<illumina::SingleEndSequencing> for Acquisition {
    fn from(acquisition: illumina::SingleEndSequencing) -> Self {
        Self::IlluminaSingleEndSequencing(acquisition)
    }
}

impl From<illumina::PairedEndSequencing> for Acquisition {
    fn from(acquisition: illumina::PairedEndSequencing) -> Self {
        Self::IlluminaPairedEndSequencing(acquisition)
    }
}
