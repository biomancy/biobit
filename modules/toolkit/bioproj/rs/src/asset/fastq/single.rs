//! Single-file FASTQ assets.

use super::super::core::AssetCore;
use super::super::uri;
use super::super::{AssetId, Uri};
use crate::provenance::run::Run as RootRun;
use crate::provenance::run::illumina::SingleEndSequencingId as SingleEndRunId;
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

/// A typed run reference accepted by a single-file FASTQ asset.
///
/// New single-file sequencing modalities can be added here without coupling
/// the FASTQ format itself to a platform name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FastqInput {
    /// A logical output of single-end Illumina sequencing.
    IlluminaSingleEndSequencing(SingleEndRunId),
}

impl FastqInput {
    /// Returns the common workspace-local ID of the referenced run.
    pub fn id(&self) -> &Id {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.as_id(),
        }
    }

    fn resolve(run_id: Id, runs: &BTreeMap<Id, RootRun>) -> Result<Self> {
        let run = runs
            .get(&run_id)
            .ok_or_else(|| eyre::eyre!("Fastq Asset references unknown Run '{run_id}'"))?;
        match run {
            RootRun::IlluminaSingleEndSequencing(run) => {
                Ok(Self::IlluminaSingleEndSequencing(run.id().clone()))
            }
            _ => bail!("Fastq Asset references Run '{run_id}', which is not compatible with Fastq"),
        }
    }

    fn validate_references(&self, runs: &BTreeMap<Id, RootRun>) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(id) => match runs.get(id.as_id()) {
                Some(RootRun::IlluminaSingleEndSequencing(_)) => Ok(()),
                Some(_) => bail!(
                    "Fastq Asset Run reference '{}' resolves to a different Run type",
                    id
                ),
                None => bail!("Fastq Asset references unknown Run '{id}'"),
            },
        }
    }
}

/// A single-file FASTQ asset.
///
/// Its non-empty `locations` set gives equivalent URI locators for the same
/// file, such as mirrors in two storage environments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fastq {
    core: AssetCore<FastqInput>,
    locations: BTreeSet<Uri>,
}

impl Fastq {
    /// Creates a single-file FASTQ asset.
    pub fn new(
        id: AssetId,
        run: FastqInput,
        locations: impl IntoIterator<Item = impl AsRef<str>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: AssetCore::new(id, run, meta, description)?,
            locations: uri::non_empty_uri_set("Fastq::locations", locations)?,
        })
    }

    fn from_parts(
        id: AssetId,
        run: FastqInput,
        locations: BTreeSet<Uri>,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: AssetCore {
                id,
                run,
                meta,
                description,
            },
            locations,
        }
    }

    /// Returns this asset's identifier.
    pub fn id(&self) -> &AssetId {
        &self.core.id
    }

    /// Returns the typed parent run.
    pub fn run(&self) -> &FastqInput {
        &self.core.run
    }

    /// Returns equivalent URI locators for this FASTQ file.
    pub fn locations(&self) -> &BTreeSet<Uri> {
        &self.locations
    }

    pub(crate) fn validate_references(&self, runs: &BTreeMap<Id, RootRun>) -> Result<()> {
        self.run().validate_references(runs)
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.core.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&str> {
        self.core.description.as_deref()
    }
}

impl AsRef<Id> for Fastq {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

impl Serialize for Fastq {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedFastq {
            id: self.id(),
            run: self.run().id(),
            locations: &self.locations,
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedFastq<'a> {
    id: &'a AssetId,
    run: &'a Id,
    locations: &'a BTreeSet<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedFastq {
    id: AssetId,
    run: Id,
    #[serde(deserialize_with = "uri::deserialize_non_empty_uri_set")]
    locations: BTreeSet<Uri>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

impl UnresolvedFastq {
    pub(crate) fn resolve(self, runs: &BTreeMap<Id, RootRun>) -> Result<Fastq> {
        Ok(Fastq::from_parts(
            self.id,
            FastqInput::resolve(self.run, runs)?,
            self.locations,
            self.meta,
            self.description,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Fastq, FastqInput};
    use crate::asset::{Asset, AssetId};
    use crate::provenance::run::illumina::SingleEndSequencingId as SingleEndRunId;

    #[test]
    fn serializes_a_generic_fastq_format() {
        let asset = Fastq::new(
            AssetId::new("AST1").unwrap(),
            FastqInput::IlluminaSingleEndSequencing(SingleEndRunId::new("RUN1").unwrap()),
            ["s3://bucket/read.fq.gz"],
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&Asset::Fastq(asset)).unwrap(),
            r#"{"type":"Fastq","id":"AST1","run":"RUN1","locations":["s3://bucket/read.fq.gz"]}"#
        );
    }
}
