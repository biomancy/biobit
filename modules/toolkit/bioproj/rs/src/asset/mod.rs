//! Physical data artifacts kept separate from biological provenance.

mod core;
pub mod fastq;
mod uri;

pub use uri::Uri;

use crate::Id;
use crate::primitives::define_entity_id;
use crate::provenance::{Provenance, Run};
use crate::validation;
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

define_entity_id!(AssetId, "The identifier of a [`crate::Asset`].");

/// A concrete data artifact format.
///
/// Each typed variant owns the set of run types that can be its parent. This
/// keeps file format compatibility independent of sequencing platform names.
/// This outer enum owns the serialized `type` discriminator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Asset {
    /// A single-file FASTQ output.
    Fastq(fastq::Fastq),
    /// A paired FASTQ output with distinct read-one and read-two files.
    PairedFastq(fastq::PairedFastq),
}

impl Asset {
    /// Returns this asset's identifier.
    pub fn id(&self) -> &AssetId {
        match self {
            Self::Fastq(asset) => asset.id(),
            Self::PairedFastq(asset) => asset.id(),
        }
    }

    fn validate_references(&self, runs: &BTreeMap<Id, Run>) -> Result<()> {
        match self {
            Self::Fastq(asset) => asset.validate_references(runs),
            Self::PairedFastq(asset) => asset.validate_references(runs),
        }
    }
}

impl AsRef<Id> for Asset {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum UnresolvedAsset {
    Fastq(fastq::UnresolvedFastq),
    PairedFastq(fastq::UnresolvedPairedFastq),
}

impl UnresolvedAsset {
    fn resolve(self, runs: &BTreeMap<Id, Run>) -> Result<Asset> {
        match self {
            Self::Fastq(asset) => Ok(Asset::Fastq(asset.resolve(runs)?)),
            Self::PairedFastq(asset) => Ok(Asset::PairedFastq(asset.resolve(runs)?)),
        }
    }
}

/// A resolved collection of assets for a provenance graph.
///
/// Construction and deserialization require the parent [`Provenance`] so raw
/// `run` IDs can be resolved into each format's typed input contract. The
/// collection can also be owned by a [`crate::Project`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assets {
    assets: BTreeMap<AssetId, Asset>,
}

impl Assets {
    /// Constructs and validates assets against their parent provenance graph.
    pub fn new(provenance: &Provenance, assets: impl IntoIterator<Item = Asset>) -> Result<Self> {
        let assets: Vec<_> = assets.into_iter().collect();
        validation::unique_ids(
            "provenance and assets",
            provenance
                .ids()
                .chain(assets.iter().map(|asset| asset.id().as_id())),
        )?;

        let assets: BTreeMap<_, _> = assets
            .into_iter()
            .map(|asset| (asset.id().clone(), asset))
            .collect();
        let result = Self { assets };
        result.validate_against(provenance)?;

        Ok(result)
    }

    /// Returns concrete assets keyed by their typed IDs.
    pub fn assets(&self) -> &BTreeMap<AssetId, Asset> {
        &self.assets
    }

    /// Finds an asset by its typed ID.
    pub fn asset(&self, id: &AssetId) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Validates this collection against its parent provenance graph.
    pub(crate) fn validate_against(&self, provenance: &Provenance) -> Result<()> {
        validation::unique_ids("provenance and assets", provenance.ids().chain(self.ids()))?;
        for asset in self.assets.values() {
            asset.validate_references(provenance.runs())?;
        }
        Ok(())
    }

    /// Iterates over IDs occupied by this asset collection.
    pub(crate) fn ids(&self) -> impl Iterator<Item = &Id> {
        self.assets.values().map(|asset| asset.id().as_id())
    }

    /// Deserializes and resolves an assets payload using parent provenance.
    ///
    /// A standalone `Deserialize` implementation would lack the runs needed
    /// to resolve raw IDs into typed file-format inputs. This explicit method
    /// provides that context when deserializing the domain on its own.
    pub fn deserialize_with_provenance<'de, D>(
        provenance: &Provenance,
        deserializer: D,
    ) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        UnresolvedAssets::deserialize(deserializer)?
            .resolve(provenance)
            .map_err(serde::de::Error::custom)
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
pub(crate) struct UnresolvedAssets {
    assets: Vec<UnresolvedAsset>,
}

impl UnresolvedAssets {
    pub(crate) fn resolve(self, provenance: &Provenance) -> Result<Assets> {
        let assets = self
            .assets
            .into_iter()
            .map(|asset| asset.resolve(provenance.runs()))
            .collect::<Result<Vec<_>>>()?;
        Assets::new(provenance, assets)
    }
}

#[cfg(test)]
mod tests {
    use super::{Asset, AssetId, Assets};
    use crate::asset::fastq::{Fastq, FastqInput};
    use crate::provenance::assay::illumina::{
        SequencingInput, SingleEndSequencing, SingleEndSequencingId,
    };
    use crate::provenance::library::illumina::{DnaLibrary, DnaLibraryId};
    use crate::provenance::run::illumina::{
        SingleEndSequencing as SingleEndRun, SingleEndSequencingId as SingleEndRunId,
    };
    use crate::provenance::{Assay, Library, Provenance, Run, Sample, SampleId, Source, SourceId};

    fn provenance() -> Provenance {
        let source_id = SourceId::new("SRC1").unwrap();
        let sample_id = SampleId::new("SMP1").unwrap();
        let library_id = DnaLibraryId::new("LIB1").unwrap();
        let assay_id = SingleEndSequencingId::new("ASY1").unwrap();
        let run_id = SingleEndRunId::new("RUN1").unwrap();

        Provenance::new(
            [Source::new(
                source_id.clone(),
                "Homo sapiens",
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap()],
            [Sample::new(
                sample_id.clone(),
                [source_id],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap()],
            [Library::IlluminaDna(
                DnaLibrary::new(
                    library_id.clone(),
                    [sample_id],
                    ["DNA"],
                    ["none"],
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
            [Assay::IlluminaSingleEndSequencing(
                SingleEndSequencing::new(
                    assay_id.clone(),
                    SequencingInput::Dna(library_id),
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
            [Run::IlluminaSingleEndSequencing(
                SingleEndRun::new(
                    run_id,
                    assay_id,
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap()
    }

    #[test]
    fn validates_fastq_against_its_typed_run() {
        let provenance = provenance();
        let assets = Assets::new(
            &provenance,
            [Asset::Fastq(
                Fastq::new(
                    AssetId::new("AST1").unwrap(),
                    FastqInput::IlluminaSingleEndSequencing(SingleEndRunId::new("RUN1").unwrap()),
                    ["s3://bucket/read.fq.gz"],
                    [("checksum", "sha256:abc")],
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(assets.assets().len(), 1);
        assert_eq!(
            serde_json::to_string(&assets).unwrap(),
            r#"{"assets":[{"type":"Fastq","id":"AST1","run":"RUN1","locations":["s3://bucket/read.fq.gz"],"meta":{"checksum":"sha256:abc"}}]}"#
        );
    }

    #[test]
    fn deserializes_fastq_with_parent_provenance() {
        let provenance = provenance();
        let mut deserializer = serde_json::Deserializer::from_str(
            r#"{
                "assets": [
                    {
                        "type": "Fastq",
                        "id": "AST1",
                        "run": "RUN1",
                        "locations": ["s3://bucket/read.fq.gz"]
                    }
                ]
            }"#,
        );
        let assets = Assets::deserialize_with_provenance(&provenance, &mut deserializer).unwrap();

        let asset = assets.asset(&AssetId::new("AST1").unwrap()).unwrap();
        let Asset::Fastq(asset) = asset else {
            panic!("serialized FASTQ resolved to a different asset type");
        };
        assert_eq!(
            asset.run(),
            &FastqInput::IlluminaSingleEndSequencing(SingleEndRunId::new("RUN1").unwrap())
        );
    }

    #[test]
    fn rejects_asset_ids_already_used_by_provenance() {
        let provenance = provenance();
        assert!(
            Assets::new(
                &provenance,
                [Asset::Fastq(
                    Fastq::new(
                        AssetId::new("RUN1").unwrap(),
                        FastqInput::IlluminaSingleEndSequencing(
                            SingleEndRunId::new("RUN1").unwrap(),
                        ),
                        ["s3://bucket/read.fq.gz"],
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                )],
            )
            .is_err()
        );
    }
}
