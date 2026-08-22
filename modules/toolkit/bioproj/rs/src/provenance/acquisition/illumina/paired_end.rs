use super::validate;
use crate::primitives::define_entity_id;
use crate::provenance::library::Library;
use crate::provenance::library::p5p7;
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

define_entity_id!(
    PairedEndSequencingId,
    "The identifier of a [`crate::provenance::acquisition::illumina::PairedEndSequencing`]."
);

/// One logical paired-end Illumina sequencing acquisition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedEndSequencing {
    id: PairedEndSequencingId,
    libraries: NonEmpty<BTreeSet<p5p7::LibraryId>>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl PairedEndSequencing {
    /// Creates a paired-end acquisition from one or more intentionally pooled libraries.
    pub fn new(
        id: PairedEndSequencingId,
        libraries: impl IntoIterator<Item = p5p7::LibraryId>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            libraries: NonEmpty::try_from_iter(libraries)?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this acquisition's identifier.
    pub fn id(&self) -> &PairedEndSequencingId {
        &self.id
    }

    /// Returns the libraries intentionally pooled into this acquisition.
    pub fn libraries(&self) -> &NonEmpty<BTreeSet<p5p7::LibraryId>> {
        &self.libraries
    }

    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        validate::validate(&self.libraries, libraries)
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
