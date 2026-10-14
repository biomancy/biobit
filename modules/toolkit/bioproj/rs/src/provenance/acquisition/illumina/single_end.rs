use super::validate;
use crate::primitives::define_entity_id;
use crate::provenance::Provenance;
use crate::provenance::library::p5p7;
use crate::{Meta, NonEmpty, OrderedSet, Result, Text};

define_entity_id!(
    SingleEndSequencingId,
    "The identifier of a [`crate::provenance::acquisition::illumina::SingleEndSequencing`]."
);

/// One logical single-end Illumina sequencing acquisition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SingleEndSequencing {
    id: SingleEndSequencingId,
    libraries: NonEmpty<OrderedSet<p5p7::LibraryId>>,
    meta: Meta,
    description: Option<Text>,
}

impl SingleEndSequencing {
    /// Creates a single-end acquisition from one or more intentionally pooled libraries.
    pub fn new(
        id: SingleEndSequencingId,
        libraries: NonEmpty<OrderedSet<p5p7::LibraryId>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            libraries,
            meta,
            description,
        }
    }

    /// Returns this acquisition's identifier.
    pub fn id(&self) -> &SingleEndSequencingId {
        &self.id
    }

    /// Returns the libraries intentionally pooled into this acquisition.
    pub fn libraries(&self) -> &NonEmpty<OrderedSet<p5p7::LibraryId>> {
        &self.libraries
    }

    pub(crate) fn validate_against(&self, provenance: &Provenance) -> Result<()> {
        validate::validate_against(self.id.as_untyped(), &self.libraries, provenance)
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }
}
