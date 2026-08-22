//! Immutable stored data artifacts.

pub mod fastq;
mod id;

pub use id::{AssetId, AssetIdRef};

use crate::{Meta, NonEmpty};
use serde::{Deserialize, Serialize};

/// A concrete immutable data artifact.
///
/// Assets describe storage only. Acquisition membership and file layout are
/// declared by [`crate::data::Dataset`] records.
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

#[cfg(test)]
mod tests {
    use super::Asset;
    use super::fastq::{Fastq, FastqId};

    #[test]
    fn assets_are_independent_storage_records() {
        let asset = Asset::Fastq(
            Fastq::new(
                FastqId::new("AST1").unwrap(),
                "file:reads.fq.gz",
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        );

        let json = serde_json::to_string(&asset).unwrap();
        assert_eq!(
            json,
            r#"{"type":"Fastq","id":"AST1","location":"file:reads.fq.gz"}"#
        );
        assert_eq!(serde_json::from_str::<Asset>(&json).unwrap(), asset);
    }
}
