//! Single-file FASTQ assets.

use crate::primitives::define_entity_id;
use crate::{Meta, NonEmpty, Uri};
use eyre::Result;
use serde::{Deserialize, Serialize};

define_entity_id!(
    FastqId,
    "The identifier of an [`crate::asset::fastq::Fastq`]."
);

/// One immutable FASTQ file at one storage location.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fastq {
    id: FastqId,
    location: Uri,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl Fastq {
    /// Creates a FASTQ asset.
    pub fn new(
        id: FastqId,
        location: impl Into<String>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            location: Uri::new(location)?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this asset's identifier.
    pub fn id(&self) -> &FastqId {
        &self.id
    }

    /// Returns this file's storage location.
    pub fn location(&self) -> &Uri {
        &self.location
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
