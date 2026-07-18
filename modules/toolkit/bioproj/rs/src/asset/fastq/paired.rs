//! Paired FASTQ assets.

use super::super::core::AssetCore;
use super::super::uri;
use super::super::{AssetId, Uri};
use crate::provenance::run::Run as RootRun;
use crate::provenance::run::illumina::PairedEndSequencingId as PairedEndRunId;
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

/// A typed run reference accepted by a paired FASTQ asset.
///
/// New paired-output sequencing modalities can be added here without coupling
/// the paired FASTQ format itself to a platform name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairedFastqInput {
    /// A logical output of paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(PairedEndRunId),
}

impl PairedFastqInput {
    /// Returns the common workspace-local ID of the referenced run.
    pub fn id(&self) -> &Id {
        match self {
            Self::IlluminaPairedEndSequencing(id) => id.as_id(),
        }
    }

    fn resolve(run_id: Id, runs: &BTreeMap<Id, RootRun>) -> Result<Self> {
        let run = runs
            .get(&run_id)
            .ok_or_else(|| eyre::eyre!("PairedFastq Asset references unknown Run '{run_id}'"))?;
        match run {
            RootRun::IlluminaPairedEndSequencing(run) => {
                Ok(Self::IlluminaPairedEndSequencing(run.id().clone()))
            }
            _ => bail!(
                "PairedFastq Asset references Run '{run_id}', which is not compatible with PairedFastq"
            ),
        }
    }

    fn validate_references(&self, runs: &BTreeMap<Id, RootRun>) -> Result<()> {
        match self {
            Self::IlluminaPairedEndSequencing(id) => match runs.get(id.as_id()) {
                Some(RootRun::IlluminaPairedEndSequencing(_)) => Ok(()),
                Some(_) => bail!(
                    "PairedFastq Asset Run reference '{}' resolves to a different Run type",
                    id
                ),
                None => bail!("PairedFastq Asset references unknown Run '{id}'"),
            },
        }
    }
}

/// A paired FASTQ asset with separate read-one and read-two file locations.
///
/// Each location set identifies equivalent URI locators for one mate file.
/// A paired asset therefore preserves the pairing relation without assigning a
/// platform-specific name to the FASTQ format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedFastq {
    core: AssetCore<PairedFastqInput>,
    read1: BTreeSet<Uri>,
    read2: BTreeSet<Uri>,
}

impl PairedFastq {
    /// Creates a paired FASTQ asset.
    pub fn new(
        id: AssetId,
        run: PairedFastqInput,
        read1: impl IntoIterator<Item = impl AsRef<str>>,
        read2: impl IntoIterator<Item = impl AsRef<str>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let read1 = uri::non_empty_uri_set("PairedFastq::read1", read1)?;
        let read2 = uri::non_empty_uri_set("PairedFastq::read2", read2)?;
        validate_distinct_mates(&read1, &read2)?;
        Ok(Self {
            core: AssetCore::new(id, run, meta, description)?,
            read1,
            read2,
        })
    }

    fn from_parts(
        id: AssetId,
        run: PairedFastqInput,
        read1: BTreeSet<Uri>,
        read2: BTreeSet<Uri>,
        meta: Meta,
        description: Option<String>,
    ) -> Result<Self> {
        validate_distinct_mates(&read1, &read2)?;
        Ok(Self {
            core: AssetCore {
                id,
                run,
                meta,
                description,
            },
            read1,
            read2,
        })
    }

    /// Returns this asset's identifier.
    pub fn id(&self) -> &AssetId {
        &self.core.id
    }

    /// Returns the typed parent run.
    pub fn run(&self) -> &PairedFastqInput {
        &self.core.run
    }

    /// Returns equivalent URI locators for the read-one file.
    pub fn read1(&self) -> &BTreeSet<Uri> {
        &self.read1
    }

    /// Returns equivalent URI locators for the read-two file.
    pub fn read2(&self) -> &BTreeSet<Uri> {
        &self.read2
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

impl AsRef<Id> for PairedFastq {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

fn validate_distinct_mates(read1: &BTreeSet<Uri>, read2: &BTreeSet<Uri>) -> Result<()> {
    if let Some(uri) = read1.intersection(read2).next() {
        bail!("PairedFastq::read1 and PairedFastq::read2 must not share URI '{uri}'");
    }
    Ok(())
}

impl Serialize for PairedFastq {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedPairedFastq {
            id: self.id(),
            run: self.run().id(),
            read1: &self.read1,
            read2: &self.read2,
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedPairedFastq<'a> {
    id: &'a AssetId,
    run: &'a Id,
    read1: &'a BTreeSet<Uri>,
    read2: &'a BTreeSet<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedPairedFastq {
    id: AssetId,
    run: Id,
    #[serde(deserialize_with = "uri::deserialize_non_empty_uri_set")]
    read1: BTreeSet<Uri>,
    #[serde(deserialize_with = "uri::deserialize_non_empty_uri_set")]
    read2: BTreeSet<Uri>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

impl UnresolvedPairedFastq {
    pub(crate) fn resolve(self, runs: &BTreeMap<Id, RootRun>) -> Result<PairedFastq> {
        PairedFastq::from_parts(
            self.id,
            PairedFastqInput::resolve(self.run, runs)?,
            self.read1,
            self.read2,
            self.meta,
            self.description,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{PairedFastq, PairedFastqInput};
    use crate::asset::{Asset, AssetId};
    use crate::provenance::run::illumina::PairedEndSequencingId as PairedEndRunId;

    #[test]
    fn requires_distinct_mate_locations() {
        assert!(
            PairedFastq::new(
                AssetId::new("AST1").unwrap(),
                PairedFastqInput::IlluminaPairedEndSequencing(PairedEndRunId::new("RUN1").unwrap()),
                ["s3://bucket/reads.fq.gz"],
                ["s3://bucket/reads.fq.gz"],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }

    #[test]
    fn serializes_with_separate_mates() {
        let asset = PairedFastq::new(
            AssetId::new("AST1").unwrap(),
            PairedFastqInput::IlluminaPairedEndSequencing(PairedEndRunId::new("RUN1").unwrap()),
            ["s3://bucket/read_1.fq.gz"],
            ["s3://bucket/read_2.fq.gz"],
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&Asset::PairedFastq(asset)).unwrap(),
            r#"{"type":"PairedFastq","id":"AST1","run":"RUN1","read1":["s3://bucket/read_1.fq.gz"],"read2":["s3://bucket/read_2.fq.gz"]}"#
        );
    }
}
