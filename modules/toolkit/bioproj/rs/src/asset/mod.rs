//! Immutable stored data artifacts.

pub mod fastq;
mod id;

pub use id::{AssetId, AssetIdRef};

use crate::validation;
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

/// A concrete immutable data artifact.
///
/// Assets describe storage only. Acquisition membership and file layout are
/// declared by [`crate::Dataset`] records.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Asset {
    /// One FASTQ file.
    Fastq(fastq::Fastq),
}

impl Asset {
    /// Returns this asset's concrete borrowed identifier.
    pub fn id(&self) -> AssetIdRef<'_> {
        match self {
            Self::Fastq(asset) => AssetIdRef::Fastq(asset.id()),
        }
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        match self {
            Self::Fastq(asset) => asset.meta(),
        }
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        match self {
            Self::Fastq(asset) => asset.description(),
        }
    }
}

impl From<fastq::Fastq> for Asset {
    fn from(asset: fastq::Fastq) -> Self {
        Self::Fastq(asset)
    }
}

/// An independently serializable collection of stored assets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assets {
    assets: BTreeMap<UntypedId, Asset>,
}

impl Assets {
    /// Constructs a collection with unique asset IDs.
    pub fn new(assets: impl IntoIterator<Item = Asset>) -> Result<Self> {
        let assets: Vec<_> = assets.into_iter().collect();
        validation::unique_ids("assets", assets.iter().map(|asset| asset.id().as_untyped()))?;
        Ok(Self {
            assets: assets
                .into_iter()
                .map(|asset| (asset.id().as_untyped().clone(), asset))
                .collect(),
        })
    }

    /// Returns assets keyed by their globally unique untyped IDs.
    pub fn assets(&self) -> &BTreeMap<UntypedId, Asset> {
        &self.assets
    }

    /// Finds an asset by its untyped ID.
    pub fn asset(&self, id: &UntypedId) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Iterates over IDs occupied by this collection.
    pub(crate) fn ids(&self) -> impl Iterator<Item = &UntypedId> {
        self.assets.keys()
    }
}

#[derive(Serialize)]
struct SerializedAssets<'a> {
    assets: Vec<&'a Asset>,
}

impl Serialize for Assets {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedAssets {
            assets: self.assets.values().collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedAssets {
    assets: Vec<Asset>,
}

impl<'de> Deserialize<'de> for Assets {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let assets = DeserializedAssets::deserialize(deserializer)?;
        Self::new(assets.assets).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{Asset, Assets};
    use crate::asset::fastq::{Fastq, FastqId};

    #[test]
    fn assets_are_independent_storage_records() {
        let assets = Assets::new([Asset::Fastq(
            Fastq::new(
                FastqId::new("AST1").unwrap(),
                "file:reads.fq.gz",
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        )])
        .unwrap();

        let json = serde_json::to_string(&assets).unwrap();
        assert_eq!(
            json,
            r#"{"assets":[{"type":"Fastq","id":"AST1","location":"file:reads.fq.gz"}]}"#
        );
        assert_eq!(serde_json::from_str::<Assets>(&json).unwrap(), assets);
    }
}
