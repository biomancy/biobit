//! Acquisition assays grouped by platform family.

pub mod illumina;

use crate::Id;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::Library;

/// A concrete acquisition assay.
///
/// Each variant owns a concrete acquisition family and points to its
/// platform-specific implementation type. This outer enum owns the
/// serialized `type` discriminator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Assay {
    /// Standard single-end Illumina sequencing.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencing),
    /// Standard paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencing),
}

impl Assay {
    /// Returns the common workspace-local identifier for this assay.
    pub fn id(&self) -> &Id {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.id().as_id(),
            Self::IlluminaPairedEndSequencing(assay) => assay.id().as_id(),
        }
    }

    /// Validates this assay's concrete input contract against known libraries.
    pub(crate) fn validate_references(&self, libraries: &BTreeMap<Id, Library>) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => assay.validate_references(libraries),
            Self::IlluminaPairedEndSequencing(assay) => assay.validate_references(libraries),
        }
    }
}

impl AsRef<Id> for Assay {
    fn as_ref(&self) -> &Id {
        self.id()
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub(crate) enum UnresolvedAssay {
    IlluminaSingleEndSequencing(illumina::UnresolvedSingleEndSequencing),
    IlluminaPairedEndSequencing(illumina::UnresolvedPairedEndSequencing),
}

impl UnresolvedAssay {
    /// Resolves a serialized assay's raw library reference into its typed input.
    pub(crate) fn resolve(self, libraries: &BTreeMap<Id, Library>) -> Result<Assay> {
        match self {
            Self::IlluminaSingleEndSequencing(assay) => Ok(Assay::IlluminaSingleEndSequencing(
                assay.resolve(libraries)?,
            )),
            Self::IlluminaPairedEndSequencing(assay) => Ok(Assay::IlluminaPairedEndSequencing(
                assay.resolve(libraries)?,
            )),
        }
    }
}
