//! Logical, demultiplexed acquisition outputs.

pub mod illumina;

use crate::Id;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::Assay;

/// A logical, demultiplexed acquisition output.
///
/// A run is a child of exactly one assay. It intentionally does not model a
/// physical flowcell, lane, instrument, or storage location; those details do
/// not determine the current logical data boundary. This outer enum owns the
/// serialized `type` discriminator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Run {
    /// Output from standard single-end Illumina sequencing.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencing),
    /// Output from standard paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencing),
}

impl Run {
    /// Returns the common workspace-local identifier for this run.
    pub fn id(&self) -> &Id {
        match self {
            Self::IlluminaSingleEndSequencing(run) => run.id().as_id(),
            Self::IlluminaPairedEndSequencing(run) => run.id().as_id(),
        }
    }

    /// Validates this run's typed assay parent against known assays.
    pub(crate) fn validate_references(&self, assays: &BTreeMap<Id, Assay>) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(run) => run.validate_references(assays),
            Self::IlluminaPairedEndSequencing(run) => run.validate_references(assays),
        }
    }
}

impl AsRef<Id> for Run {
    fn as_ref(&self) -> &Id {
        self.id()
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub(crate) enum UnresolvedRun {
    IlluminaSingleEndSequencing(illumina::UnresolvedSingleEndSequencing),
    IlluminaPairedEndSequencing(illumina::UnresolvedPairedEndSequencing),
}

impl UnresolvedRun {
    /// Resolves a serialized run's raw assay reference into its typed parent.
    pub(crate) fn resolve(self, assays: &BTreeMap<Id, Assay>) -> Result<Run> {
        match self {
            Self::IlluminaSingleEndSequencing(run) => {
                Ok(Run::IlluminaSingleEndSequencing(run.resolve(assays)?))
            }
            Self::IlluminaPairedEndSequencing(run) => {
                Ok(Run::IlluminaPairedEndSequencing(run.resolve(assays)?))
            }
        }
    }
}
