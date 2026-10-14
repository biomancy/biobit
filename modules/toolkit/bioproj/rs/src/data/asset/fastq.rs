//! Single-file FASTQ assets.

use crate::primitives::define_entity_id;
use crate::{FilePath, Meta, Text};

define_entity_id!(
    FastqId,
    "The identifier of an [`crate::data::asset::fastq::Fastq`]."
);

/// One immutable FASTQ file at one storage location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fastq {
    id: FastqId,
    location: FilePath,
    meta: Meta,
    description: Option<Text>,
}

impl Fastq {
    /// Creates a FASTQ asset.
    pub fn new(id: FastqId, location: FilePath, meta: Meta, description: Option<Text>) -> Self {
        Self {
            id,
            location,
            meta,
            description,
        }
    }

    /// Returns this asset's identifier.
    pub fn id(&self) -> &FastqId {
        &self.id
    }

    /// Returns this file's storage location.
    pub fn location(&self) -> &FilePath {
        &self.location
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
