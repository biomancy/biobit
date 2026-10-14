//! JSON 0.0.1 wire representations for the data domain.
//!
//! Loading attaches the manifest root to Assets and passes parsed fields
//! through domain constructors. Saving renders Asset locations against the
//! destination manifest root.

use crate::data::asset::fastq::{Fastq, FastqId};
use crate::data::dataset::fastq::{
    Pair, Paired, PairedId, PairedInput, Single, SingleId, SingleInput,
};
use crate::data::{Asset, Data, Dataset};
use crate::provenance::acquisition::illumina::{PairedEndSequencingId, SingleEndSequencingId};
use crate::{FilePath, Meta, NonEmpty, OrderedSet, Result, Root, Text};
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum AssetFromWire {
    Fastq {
        id: FastqId,
        location: PathBuf,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
}

impl AssetFromWire {
    fn resolve(self, root: &Root) -> Result<Asset> {
        match self {
            Self::Fastq {
                id,
                location,
                meta,
                description,
            } => {
                let location = if location.is_absolute() {
                    FilePath::from_absolute(location)?
                } else {
                    FilePath::from_relative(location, root.clone())?
                };
                Ok(Fastq::new(id, location, meta, description).into())
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum DatasetFromWire {
    Fastq {
        id: SingleId,
        acquisition: SingleEndSequencingId,
        assets: NonEmpty<OrderedSet<FastqId>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
    PairedFastq {
        id: PairedId,
        acquisition: PairedEndSequencingId,
        pairs: NonEmpty<OrderedSet<[FastqId; 2]>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
}

impl DatasetFromWire {
    fn resolve(self) -> Result<Dataset> {
        Ok(match self {
            Self::Fastq {
                id,
                acquisition,
                assets,
                meta,
                description,
            } => Single::new(id, acquisition, assets, meta, description).into(),
            Self::PairedFastq {
                id,
                acquisition,
                pairs,
                meta,
                description,
            } => {
                let pairs = pairs
                    .into_iter()
                    .map(|[read1, read2]| Pair::new(read1, read2))
                    .collect::<Result<Vec<_>>>()?;
                Paired::new(
                    id,
                    acquisition,
                    NonEmpty::try_from_iter(pairs)?,
                    meta,
                    description,
                )?
                .into()
            }
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DataFromWire {
    assets: Vec<AssetFromWire>,
    datasets: Vec<DatasetFromWire>,
}

impl DataFromWire {
    pub(super) fn resolve(self, root: &Root) -> Result<Data> {
        let datasets = self
            .datasets
            .into_iter()
            .map(DatasetFromWire::resolve)
            .collect::<Result<Vec<_>>>()?;
        let assets = self
            .assets
            .into_iter()
            .map(|asset| asset.resolve(root))
            .collect::<Result<Vec<_>>>()?;
        Data::new(assets, datasets)
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AssetToWire<'a> {
    Fastq {
        id: &'a FastqId,
        location: PathBuf,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
}

impl<'a> AssetToWire<'a> {
    fn new(asset: &'a Asset, destination: &Root) -> Result<Self> {
        match asset {
            Asset::Fastq(asset) => Ok(Self::Fastq {
                id: asset.id(),
                location: asset.location().rebase(destination)?,
                meta: asset.meta(),
                description: asset.description(),
            }),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum DatasetToWire<'a> {
    Fastq {
        id: &'a SingleId,
        acquisition: &'a SingleEndSequencingId,
        assets: &'a NonEmpty<OrderedSet<FastqId>>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
    PairedFastq {
        id: &'a PairedId,
        acquisition: &'a PairedEndSequencingId,
        pairs: PairsToWire<'a>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
}

impl<'a> DatasetToWire<'a> {
    fn new(dataset: &'a Dataset) -> Self {
        match dataset {
            Dataset::Fastq(dataset) => Self::Fastq {
                id: dataset.id(),
                acquisition: match dataset.acquisition() {
                    SingleInput::IlluminaSingleEndSequencing(id) => id,
                },
                assets: dataset.assets(),
                meta: dataset.meta(),
                description: dataset.description(),
            },
            Dataset::PairedFastq(dataset) => Self::PairedFastq {
                id: dataset.id(),
                acquisition: match dataset.acquisition() {
                    PairedInput::IlluminaPairedEndSequencing(id) => id,
                },
                pairs: PairsToWire(dataset.pairs()),
                meta: dataset.meta(),
                description: dataset.description(),
            },
        }
    }
}

struct PairsToWire<'a>(&'a NonEmpty<OrderedSet<Pair>>);

impl Serialize for PairsToWire<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter().map(|pair| (pair.read1(), pair.read2())))
    }
}

#[derive(Serialize)]
pub(super) struct DataToWire<'a> {
    assets: Vec<AssetToWire<'a>>,
    datasets: Vec<DatasetToWire<'a>>,
}

impl<'a> DataToWire<'a> {
    pub(super) fn new(data: &'a Data, destination: &Root) -> Result<Self> {
        Ok(Self {
            assets: data
                .assets()
                .map(|asset| AssetToWire::new(asset, destination))
                .collect::<Result<Vec<_>>>()?,
            datasets: data.datasets().map(DatasetToWire::new).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetFromWire, AssetToWire, DatasetFromWire, DatasetToWire};
    use crate::data::asset::fastq::{Fastq, FastqId};
    use crate::data::dataset::fastq::{Pair, Paired, PairedId};
    use crate::data::{Asset, Dataset};
    use crate::provenance::acquisition::illumina::PairedEndSequencingId;
    use crate::{FilePath, NonEmpty, Root};

    #[test]
    fn round_trips_a_flat_asset_with_its_root() {
        let root = Root::new(std::env::temp_dir()).unwrap();
        let asset: Asset = Fastq::new(
            FastqId::new("AST1").unwrap(),
            FilePath::from_relative("reads.fq.gz", root.clone()).unwrap(),
            Default::default(),
            None,
        )
        .into();

        let json = serde_json::to_string(&AssetToWire::new(&asset, &root).unwrap()).unwrap();
        assert_eq!(
            json,
            r#"{"type":"Fastq","id":"AST1","location":"reads.fq.gz"}"#
        );
        let from_wire: AssetFromWire = serde_json::from_str(&json).unwrap();
        assert_eq!(from_wire.resolve(&root).unwrap(), asset);
    }

    #[test]
    fn loading_validates_asset_paths_after_attaching_the_root() {
        let root = Root::new(std::env::temp_dir()).unwrap();
        let wire: AssetFromWire =
            serde_json::from_str(r#"{"type":"Fastq","id":"AST1","location":"reads/."}"#).unwrap();
        assert!(wire.resolve(&root).is_err());
    }

    #[test]
    fn preserves_ordered_fastq_pairs() {
        let pair = Pair::new(FastqId::new("B").unwrap(), FastqId::new("A").unwrap()).unwrap();
        let dataset: Dataset = Paired::new(
            PairedId::new("DATA1").unwrap(),
            PairedEndSequencingId::new("ACQ1").unwrap(),
            NonEmpty::try_from_iter([pair]).unwrap(),
            Default::default(),
            None,
        )
        .unwrap()
        .into();

        let json = serde_json::to_string(&DatasetToWire::new(&dataset)).unwrap();
        assert_eq!(
            json,
            r#"{"type":"PairedFastq","id":"DATA1","acquisition":"ACQ1","pairs":[["B","A"]]}"#
        );
        let from_wire: DatasetFromWire = serde_json::from_str(&json).unwrap();
        assert_eq!(from_wire.resolve().unwrap(), dataset);
    }
}
