use super::{id::AcquisitionIdRef, illumina};
use crate::provenance::Library;
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A concrete logical acquisition.
///
/// This initial type intentionally combines two concepts: the reusable assay
/// specification and one execution of that assay. A future model may separate
/// them so repeated acquisitions can reference one shared method definition.
#[derive(
    Clone, Debug, Eq, PartialEq, Serialize, Deserialize, kinded::Kinded, derive_more::From,
)]
#[kinded(
    kind = AcquisitionKind,
    skip_derive(From, FromStr),
    derive(Hash),
    attrs(doc = "The concrete type of an acquisition or acquisition identifier.")
)]
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
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.id().into(),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.id().into(),
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
