use crate::data::asset::fastq::FastqId;
use crate::primitives::define_entity_id;
use crate::provenance::acquisition::illumina::PairedEndSequencingId;
use crate::provenance::{AcquisitionIdRef, Provenance};
use crate::{Meta, NonEmpty};
use eyre::{Result, bail, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;

define_entity_id!(
    PairedId,
    "The identifier of a [`crate::data::dataset::fastq::Paired`] dataset."
);

/// An acquisition compatible with an ordered-pair FASTQ dataset.
///
/// This dataset-owned contract can grow to include other paired FASTQ
/// producers without coupling [`crate::data::Asset`] to acquisition semantics.
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
pub enum PairedInput {
    /// Paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(PairedEndSequencingId),
}

impl PairedInput {
    /// Returns the concrete borrowed acquisition identifier.
    pub fn id(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaPairedEndSequencing(id) => id.into(),
        }
    }

    pub(crate) fn validate(&self, provenance: &Provenance) -> Result<()> {
        match self {
            Self::IlluminaPairedEndSequencing(id) => match provenance.get(id) {
                Some(Ok(_)) => Ok(()),
                Some(Err(_)) => bail!(
                    "Paired FASTQ Dataset Acquisition reference '{}' resolves to an incompatible Acquisition type",
                    id.as_untyped()
                ),
                None => bail!(
                    "Paired FASTQ Dataset references unknown Acquisition '{}'",
                    id.as_untyped()
                ),
            },
        }
    }
}

impl Serialize for PairedInput {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id().as_untyped().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PairedInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        PairedEndSequencingId::deserialize(deserializer).map(Self::IlluminaPairedEndSequencing)
    }
}

/// One ordered read-one/read-two correspondence.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pair(FastqId, FastqId);

impl Pair {
    /// Creates an ordered pair of distinct FASTQ assets.
    pub fn new(read1: FastqId, read2: FastqId) -> Result<Self> {
        ensure!(
            read1 != read2,
            "A paired FASTQ entry must use distinct Assets"
        );
        Ok(Self(read1, read2))
    }

    /// Returns the read-one FASTQ asset ID.
    pub fn read1(&self) -> &FastqId {
        &self.0
    }

    /// Returns the read-two FASTQ asset ID.
    pub fn read2(&self) -> &FastqId {
        &self.1
    }
}

impl Serialize for Pair {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&self.0, &self.1).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Pair {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (read1, read2) = <(FastqId, FastqId)>::deserialize(deserializer)?;
        Self::new(read1, read2).map_err(serde::de::Error::custom)
    }
}

/// A complete representation made from ordered FASTQ pairs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Paired {
    id: PairedId,
    acquisition: PairedInput,
    pairs: NonEmpty<BTreeSet<Pair>>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl Paired {
    /// Creates a paired FASTQ dataset.
    pub fn new(
        id: PairedId,
        acquisition: impl Into<PairedInput>,
        pairs: impl IntoIterator<Item = (FastqId, FastqId)>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let pairs = pairs
            .into_iter()
            .map(|(read1, read2)| Pair::new(read1, read2))
            .collect::<Result<Vec<_>>>()?;
        let pairs = NonEmpty::try_from_iter(pairs)?;
        validate_unique_assets(&pairs)?;
        Ok(Self {
            id,
            acquisition: acquisition.into(),
            pairs,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    pub(crate) fn from_parts(
        id: PairedId,
        acquisition: PairedInput,
        pairs: NonEmpty<BTreeSet<Pair>>,
        meta: Meta,
        description: Option<NonEmpty<String>>,
    ) -> Result<Self> {
        validate_unique_assets(&pairs)?;
        Ok(Self {
            id,
            acquisition,
            pairs,
            meta,
            description,
        })
    }

    /// Returns this dataset's identifier.
    pub fn id(&self) -> &PairedId {
        &self.id
    }

    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> &PairedInput {
        &self.acquisition
    }

    pub(crate) fn validate(&self, provenance: &Provenance) -> Result<()> {
        self.acquisition.validate(provenance)
    }

    /// Returns the ordered FASTQ pairs.
    pub fn pairs(&self) -> &NonEmpty<BTreeSet<Pair>> {
        &self.pairs
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

fn validate_unique_assets(pairs: &NonEmpty<BTreeSet<Pair>>) -> Result<()> {
    let mut assets = BTreeSet::new();
    for pair in pairs {
        for id in [pair.read1(), pair.read2()] {
            if !assets.insert(id) {
                bail!("FASTQ Asset '{id}' appears more than once in a paired Dataset");
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedPaired {
    id: PairedId,
    acquisition: PairedInput,
    pairs: NonEmpty<BTreeSet<Pair>>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<NonEmpty<String>>,
}

impl<'de> Deserialize<'de> for Paired {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = DeserializedPaired::deserialize(deserializer)?;
        Paired::from_parts(
            value.id,
            value.acquisition,
            value.pairs,
            value.meta,
            value.description,
        )
        .map_err(serde::de::Error::custom)
    }
}
