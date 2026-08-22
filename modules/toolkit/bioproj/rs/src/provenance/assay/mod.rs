//! Acquisition assays grouped by platform family.

pub mod illumina;

use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::Library;

/// A concrete acquisition assay.
///
/// Each variant owns a concrete acquisition family and points to its
/// platform-specific implementation type. This outer enum owns the
/// serialized `type` discriminator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Assay {
    /// Single-end Illumina sequencing.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencing),
    /// Paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencing),
}

impl Assay {
    /// Returns the untyped workspace-local identifier for this assay.
    pub fn untyped_id(&self) -> &UntypedId {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.id().as_untyped(),
            Self::IlluminaPairedEndSequencing(assay) => assay.id().as_untyped(),
        }
    }

    /// Returns the untyped identifier of this assay's parent library.
    pub fn library(&self) -> &UntypedId {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.library().as_untyped(),
            Self::IlluminaPairedEndSequencing(assay) => assay.library().as_untyped(),
        }
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.meta(),
            Self::IlluminaPairedEndSequencing(assay) => assay.meta(),
        }
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.description(),
            Self::IlluminaPairedEndSequencing(assay) => assay.description(),
        }
    }

    /// Validates this assay's concrete input contract against known libraries.
    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.validate(libraries),
            Self::IlluminaPairedEndSequencing(assay) => assay.validate(libraries),
        }
    }
}
