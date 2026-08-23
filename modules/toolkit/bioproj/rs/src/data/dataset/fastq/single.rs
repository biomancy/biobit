use crate::data::asset::fastq::FastqId;
use crate::primitives::define_entity_id;
use crate::provenance::acquisition::illumina::SingleEndSequencingId;
use crate::provenance::{AcquisitionIdRef, Provenance};
use crate::{Meta, NonEmpty};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeSet;

define_entity_id!(
    SingleId,
    "The identifier of a [`crate::data::dataset::fastq::Single`] dataset."
);

/// An acquisition compatible with an independent-file FASTQ dataset.
///
/// This dataset-owned contract can grow to include other single-file FASTQ
/// producers without coupling [`crate::data::Asset`] to acquisition semantics.
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
pub enum SingleInput {
    /// Single-end Illumina sequencing.
    IlluminaSingleEndSequencing(SingleEndSequencingId),
}

impl SingleInput {
    /// Returns the concrete borrowed acquisition identifier.
    pub fn id(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.into(),
        }
    }

    pub(crate) fn validate(&self, provenance: &Provenance) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(id) => match provenance.get(id) {
                Some(Ok(_)) => Ok(()),
                Some(Err(_)) => bail!(
                    "FASTQ Dataset Acquisition reference '{}' resolves to an incompatible Acquisition type",
                    id.as_untyped()
                ),
                None => bail!(
                    "FASTQ Dataset references unknown Acquisition '{}'",
                    id.as_untyped()
                ),
            },
        }
    }
}

impl Serialize for SingleInput {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id().as_untyped().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SingleInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SingleEndSequencingId::deserialize(deserializer).map(Self::IlluminaSingleEndSequencing)
    }
}

/// A complete representation made from independent FASTQ files.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Single {
    id: SingleId,
    acquisition: SingleInput,
    assets: NonEmpty<BTreeSet<FastqId>>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl Single {
    /// Creates a FASTQ dataset.
    pub fn new(
        id: SingleId,
        acquisition: impl Into<SingleInput>,
        assets: impl IntoIterator<Item = FastqId>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            acquisition: acquisition.into(),
            assets: NonEmpty::try_from_iter(assets)?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this dataset's identifier.
    pub fn id(&self) -> &SingleId {
        &self.id
    }

    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> &SingleInput {
        &self.acquisition
    }

    pub(crate) fn validate(&self, provenance: &Provenance) -> Result<()> {
        self.acquisition.validate(provenance)
    }

    /// Returns the complete set of FASTQ assets.
    pub fn assets(&self) -> &NonEmpty<BTreeSet<FastqId>> {
        &self.assets
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
