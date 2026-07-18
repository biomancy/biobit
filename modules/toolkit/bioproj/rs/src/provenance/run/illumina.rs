//! Logical outputs of Illumina sequencing assays.

use crate::primitives::define_entity_id;
use crate::provenance::assay::Assay as RootAssay;
use crate::provenance::assay::illumina::{
    PairedEndSequencingId as PairedEndAssayId, SingleEndSequencingId as SingleEndAssayId,
};
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;

define_entity_id!(
    SingleEndSequencingId,
    "The identifier of a [`crate::provenance::run::illumina::SingleEndSequencing`]."
);
define_entity_id!(
    PairedEndSequencingId,
    "The identifier of a [`crate::provenance::run::illumina::PairedEndSequencing`]."
);

/// Logical, demultiplexed output of a single-end Illumina sequencing assay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SingleEndSequencing {
    core: RunCore<SingleEndSequencingId, SingleEndAssayId>,
}

impl SingleEndSequencing {
    /// Creates a single-end Illumina sequencing run.
    pub fn new(
        id: SingleEndSequencingId,
        assay: SingleEndAssayId,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: RunCore::new(id, assay, meta, description)?,
        })
    }

    fn from_parts(
        id: SingleEndSequencingId,
        assay: SingleEndAssayId,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: RunCore {
                id,
                assay,
                meta,
                description,
            },
        }
    }

    /// Returns this run's identifier.
    pub fn id(&self) -> &SingleEndSequencingId {
        &self.core.id
    }

    /// Returns the typed parent assay.
    pub fn assay(&self) -> &SingleEndAssayId {
        &self.core.assay
    }

    pub(crate) fn validate_references(&self, assays: &BTreeMap<Id, RootAssay>) -> Result<()> {
        match assays.get(self.assay().as_id()) {
            Some(RootAssay::IlluminaSingleEndSequencing(_)) => Ok(()),
            Some(_) => bail!(
                "Illumina single-end Run '{}' references a different Assay type",
                self.id()
            ),
            None => bail!(
                "Illumina single-end Run '{}' references unknown Assay '{}'",
                self.id(),
                self.assay()
            ),
        }
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

impl AsRef<Id> for SingleEndSequencing {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

/// Logical, demultiplexed output of a paired-end Illumina sequencing assay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedEndSequencing {
    core: RunCore<PairedEndSequencingId, PairedEndAssayId>,
}

impl PairedEndSequencing {
    /// Creates a paired-end Illumina sequencing run.
    pub fn new(
        id: PairedEndSequencingId,
        assay: PairedEndAssayId,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: RunCore::new(id, assay, meta, description)?,
        })
    }

    fn from_parts(
        id: PairedEndSequencingId,
        assay: PairedEndAssayId,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: RunCore {
                id,
                assay,
                meta,
                description,
            },
        }
    }

    /// Returns this run's identifier.
    pub fn id(&self) -> &PairedEndSequencingId {
        &self.core.id
    }

    /// Returns the typed parent assay.
    pub fn assay(&self) -> &PairedEndAssayId {
        &self.core.assay
    }

    pub(crate) fn validate_references(&self, assays: &BTreeMap<Id, RootAssay>) -> Result<()> {
        match assays.get(self.assay().as_id()) {
            Some(RootAssay::IlluminaPairedEndSequencing(_)) => Ok(()),
            Some(_) => bail!(
                "Illumina paired-end Run '{}' references a different Assay type",
                self.id()
            ),
            None => bail!(
                "Illumina paired-end Run '{}' references unknown Assay '{}'",
                self.id(),
                self.assay()
            ),
        }
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

impl AsRef<Id> for PairedEndSequencing {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RunCore<I, A> {
    id: I,
    assay: A,
    meta: Meta,
    description: Option<String>,
}

impl<I, A> RunCore<I, A> {
    fn new(
        id: I,
        assay: A,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            assay,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }
}

impl Serialize for SingleEndSequencing {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedSingleEndSequencing {
            id: self.id(),
            assay: self.assay(),
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

impl Serialize for PairedEndSequencing {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedPairedEndSequencing {
            id: self.id(),
            assay: self.assay(),
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedSingleEndSequencing<'a> {
    id: &'a SingleEndSequencingId,
    assay: &'a SingleEndAssayId,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Serialize)]
struct SerializedPairedEndSequencing<'a> {
    id: &'a PairedEndSequencingId,
    assay: &'a PairedEndAssayId,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedSingleEndSequencing {
    id: SingleEndSequencingId,
    assay: Id,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedPairedEndSequencing {
    id: PairedEndSequencingId,
    assay: Id,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

impl UnresolvedSingleEndSequencing {
    pub(crate) fn resolve(self, assays: &BTreeMap<Id, RootAssay>) -> Result<SingleEndSequencing> {
        let assay = assays
            .get(&self.assay)
            .ok_or_else(|| eyre::eyre!("Run references unknown Assay '{}'", self.assay))?;
        let RootAssay::IlluminaSingleEndSequencing(assay) = assay else {
            bail!(
                "Illumina single-end Run '{}' references a different Assay type",
                self.id
            );
        };
        Ok(SingleEndSequencing::from_parts(
            self.id,
            assay.id().clone(),
            self.meta,
            self.description,
        ))
    }
}

impl UnresolvedPairedEndSequencing {
    pub(crate) fn resolve(self, assays: &BTreeMap<Id, RootAssay>) -> Result<PairedEndSequencing> {
        let assay = assays
            .get(&self.assay)
            .ok_or_else(|| eyre::eyre!("Run references unknown Assay '{}'", self.assay))?;
        let RootAssay::IlluminaPairedEndSequencing(assay) = assay else {
            bail!(
                "Illumina paired-end Run '{}' references a different Assay type",
                self.id
            );
        };
        Ok(PairedEndSequencing::from_parts(
            self.id,
            assay.id().clone(),
            self.meta,
            self.description,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{SingleEndSequencing, SingleEndSequencingId};
    use crate::provenance::Run;
    use crate::provenance::assay::illumina::SingleEndSequencingId as SingleEndAssayId;

    #[test]
    fn serializes_the_assay_reference_without_repeating_its_type() {
        let run = SingleEndSequencing::new(
            SingleEndSequencingId::new("RUN1").unwrap(),
            SingleEndAssayId::new("ASY1").unwrap(),
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&Run::IlluminaSingleEndSequencing(run)).unwrap(),
            r#"{"type":"IlluminaSingleEndSequencing","id":"RUN1","assay":"ASY1"}"#
        );
    }
}
