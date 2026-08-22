use super::validate;
use crate::primitives::define_entity_id;
use crate::provenance::library::Library;
use crate::provenance::library::p5p7::LibraryId;
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

define_entity_id!(
    PairedEndSequencingId,
    "The identifier of a [`crate::provenance::assay::illumina::PairedEndSequencing`]."
);

/// Standard paired-end Illumina sequencing of a compatible library.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedEndSequencing {
    id: PairedEndSequencingId,
    library: LibraryId,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl PairedEndSequencing {
    /// Creates a paired-end Illumina sequencing assay.
    pub fn new(
        id: PairedEndSequencingId,
        library: LibraryId,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            library,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this assay's identifier.
    pub fn id(&self) -> &PairedEndSequencingId {
        &self.id
    }

    /// Returns this assay's typed library input.
    pub fn library(&self) -> &LibraryId {
        &self.library
    }

    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        validate::validate(self.library(), libraries)
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        self.description.as_ref()
    }
}
