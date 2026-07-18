//! Illumina-compatible library preparations.

use super::strandedness::Strandedness;
use crate::primitives::{NonEmptyIdSet, define_entity_id};
use crate::validation;
use crate::{Id, Meta, MetaVal};
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;

use crate::provenance::SampleId;

define_entity_id!(
    DnaLibraryId,
    "The identifier of a [`crate::provenance::library::illumina::DnaLibrary`]."
);
define_entity_id!(
    CdnaLibraryId,
    "The identifier of a [`crate::provenance::library::illumina::CdnaLibrary`]."
);

/// An Illumina-compatible library prepared from DNA.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnaLibrary {
    core: LibraryCore<DnaLibraryId>,
}

impl DnaLibrary {
    /// Creates an Illumina-compatible DNA library.
    pub fn new(
        id: DnaLibraryId,
        samples: impl IntoIterator<Item = SampleId>,
        molecule: impl IntoIterator<Item = impl Into<String>>,
        selection: impl IntoIterator<Item = impl Into<String>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: LibraryCore::new(id, samples, molecule, selection, meta, description)?,
        })
    }

    fn from_parts(
        id: DnaLibraryId,
        samples: NonEmptyIdSet<SampleId>,
        molecule: BTreeSet<String>,
        selection: BTreeSet<String>,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: LibraryCore {
                id,
                samples,
                molecule,
                selection,
                meta,
                description,
            },
        }
    }

    /// Returns this library's identifier.
    pub fn id(&self) -> &DnaLibraryId {
        &self.core.id
    }

    /// Returns the IDs of this library's parent samples.
    pub fn samples(&self) -> &BTreeSet<SampleId> {
        self.core.samples.as_set()
    }

    /// Returns the molecular material in this library.
    pub fn molecule(&self) -> &BTreeSet<String> {
        &self.core.molecule
    }

    /// Returns the selection methods used for this library.
    pub fn selection(&self) -> &BTreeSet<String> {
        &self.core.selection
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

impl AsRef<Id> for DnaLibrary {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

/// An Illumina-compatible library prepared from RNA-derived cDNA.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdnaLibrary {
    core: LibraryCore<CdnaLibraryId>,
    strandedness: Strandedness,
}

impl CdnaLibrary {
    /// Creates an Illumina-compatible RNA-derived cDNA library.
    pub fn new(
        id: CdnaLibraryId,
        samples: impl IntoIterator<Item = SampleId>,
        molecule: impl IntoIterator<Item = impl Into<String>>,
        selection: impl IntoIterator<Item = impl Into<String>>,
        strandedness: Strandedness,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: LibraryCore::new(id, samples, molecule, selection, meta, description)?,
            strandedness,
        })
    }

    fn from_parts(
        id: CdnaLibraryId,
        samples: NonEmptyIdSet<SampleId>,
        molecule: BTreeSet<String>,
        selection: BTreeSet<String>,
        strandedness: Strandedness,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: LibraryCore {
                id,
                samples,
                molecule,
                selection,
                meta,
                description,
            },
            strandedness,
        }
    }

    /// Returns this library's identifier.
    pub fn id(&self) -> &CdnaLibraryId {
        &self.core.id
    }

    /// Returns the IDs of this library's parent samples.
    pub fn samples(&self) -> &BTreeSet<SampleId> {
        self.core.samples.as_set()
    }

    /// Returns the molecular material in this library.
    pub fn molecule(&self) -> &BTreeSet<String> {
        &self.core.molecule
    }

    /// Returns the selection methods used for this library.
    pub fn selection(&self) -> &BTreeSet<String> {
        &self.core.selection
    }

    /// Returns the RNA strand specificity of this library preparation.
    pub fn strandedness(&self) -> Strandedness {
        self.strandedness
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

impl AsRef<Id> for CdnaLibrary {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LibraryCore<I> {
    id: I,
    samples: NonEmptyIdSet<SampleId>,
    molecule: BTreeSet<String>,
    selection: BTreeSet<String>,
    meta: Meta,
    description: Option<String>,
}

impl<I> LibraryCore<I> {
    fn new(
        id: I,
        samples: impl IntoIterator<Item = SampleId>,
        molecule: impl IntoIterator<Item = impl Into<String>>,
        selection: impl IntoIterator<Item = impl Into<String>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            samples: NonEmptyIdSet::new("Library::samples", samples)?,
            molecule: validation::non_empty_string_set("Library::molecule", molecule)?,
            selection: validation::non_empty_string_set("Library::selection", selection)?,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }
}

impl Serialize for DnaLibrary {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedDnaLibrary {
            id: self.id(),
            samples: &self.core.samples,
            molecule: &self.core.molecule,
            selection: &self.core.selection,
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

impl Serialize for CdnaLibrary {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedCdnaLibrary {
            id: self.id(),
            samples: &self.core.samples,
            molecule: &self.core.molecule,
            selection: &self.core.selection,
            strandedness: self.strandedness,
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedDnaLibrary<'a> {
    id: &'a DnaLibraryId,
    samples: &'a NonEmptyIdSet<SampleId>,
    molecule: &'a BTreeSet<String>,
    selection: &'a BTreeSet<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Serialize)]
struct SerializedCdnaLibrary<'a> {
    id: &'a CdnaLibraryId,
    samples: &'a NonEmptyIdSet<SampleId>,
    molecule: &'a BTreeSet<String>,
    selection: &'a BTreeSet<String>,
    strandedness: Strandedness,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedDnaLibrary {
    id: DnaLibraryId,
    samples: NonEmptyIdSet<SampleId>,
    #[serde(deserialize_with = "validation::deserialize_non_empty_string_set")]
    molecule: BTreeSet<String>,
    #[serde(deserialize_with = "validation::deserialize_non_empty_string_set")]
    selection: BTreeSet<String>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedCdnaLibrary {
    id: CdnaLibraryId,
    samples: NonEmptyIdSet<SampleId>,
    #[serde(deserialize_with = "validation::deserialize_non_empty_string_set")]
    molecule: BTreeSet<String>,
    #[serde(deserialize_with = "validation::deserialize_non_empty_string_set")]
    selection: BTreeSet<String>,
    strandedness: Strandedness,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

impl From<DeserializedDnaLibrary> for DnaLibrary {
    fn from(value: DeserializedDnaLibrary) -> Self {
        Self::from_parts(
            value.id,
            value.samples,
            value.molecule,
            value.selection,
            value.meta,
            value.description,
        )
    }
}

impl From<DeserializedCdnaLibrary> for CdnaLibrary {
    fn from(value: DeserializedCdnaLibrary) -> Self {
        Self::from_parts(
            value.id,
            value.samples,
            value.molecule,
            value.selection,
            value.strandedness,
            value.meta,
            value.description,
        )
    }
}

impl<'de> Deserialize<'de> for DnaLibrary {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DeserializedDnaLibrary::deserialize(deserializer)?.into())
    }
}

impl<'de> Deserialize<'de> for CdnaLibrary {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DeserializedCdnaLibrary::deserialize(deserializer)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{CdnaLibrary, CdnaLibraryId, DnaLibrary, DnaLibraryId};
    use crate::provenance::SampleId;
    use crate::provenance::library::strandedness::Strandedness;

    fn sample_id(id: &str) -> SampleId {
        SampleId::new(id).unwrap()
    }

    #[test]
    fn dna_and_cdna_libraries_have_distinct_typed_ids() {
        let dna = DnaLibrary::new(
            DnaLibraryId::new("LIB_DNA").unwrap(),
            [sample_id("SMP1")],
            ["DNA"],
            ["none"],
            [("kit", "DNA Prep")],
            None::<String>,
        )
        .unwrap();
        let cdna = CdnaLibrary::new(
            CdnaLibraryId::new("LIB_CDNA").unwrap(),
            [sample_id("SMP1")],
            ["cDNA"],
            ["poly-A"],
            Strandedness::Unknown,
            [("kit", "RNA Prep")],
            None::<String>,
        )
        .unwrap();

        assert_eq!(dna.id().as_str(), "LIB_DNA");
        assert_eq!(cdna.strandedness(), Strandedness::Unknown);
    }

    #[test]
    fn deserialization_requires_explicit_cdna_strandedness() {
        assert!(
            serde_json::from_str::<CdnaLibrary>(
                r#"{
                    "id": "LIB1",
                    "samples": ["SMP1"],
                    "molecule": ["cDNA"],
                    "selection": ["poly-A"]
                }"#,
            )
            .is_err()
        );
    }
}
