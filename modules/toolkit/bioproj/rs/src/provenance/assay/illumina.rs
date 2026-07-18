//! Illumina sequencing assays.

use crate::primitives::define_entity_id;
use crate::provenance::library::Library as RootLibrary;
use crate::provenance::library::illumina::{CdnaLibraryId, DnaLibraryId};
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;

define_entity_id!(
    SingleEndSequencingId,
    "The identifier of a [`crate::provenance::assay::illumina::SingleEndSequencing`]."
);
define_entity_id!(
    PairedEndSequencingId,
    "The identifier of a [`crate::provenance::assay::illumina::PairedEndSequencing`]."
);

/// A typed library reference accepted by standard Illumina sequencing assays.
///
/// This union is an assay-owned input contract: both DNA and RNA-derived cDNA
/// libraries can be sequenced with standard Illumina single- or paired-end
/// acquisition, while other future library kinds are not accepted implicitly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SequencingInput {
    /// An Illumina-compatible DNA library.
    Dna(DnaLibraryId),
    /// An Illumina-compatible RNA-derived cDNA library.
    Cdna(CdnaLibraryId),
}

impl SequencingInput {
    /// Returns the common workspace-local ID of the referenced library.
    pub fn id(&self) -> &Id {
        match self {
            Self::Dna(id) => id.as_id(),
            Self::Cdna(id) => id.as_id(),
        }
    }

    fn resolve(library_id: Id, libraries: &BTreeMap<Id, RootLibrary>) -> Result<Self> {
        let library = libraries
            .get(&library_id)
            .ok_or_else(|| eyre::eyre!("Assay references unknown Library '{library_id}'"))?;

        match library {
            RootLibrary::IlluminaDna(library) => Ok(Self::Dna(library.id().clone())),
            RootLibrary::IlluminaCdna(library) => Ok(Self::Cdna(library.id().clone())),
        }
    }

    fn validate_references(&self, libraries: &BTreeMap<Id, RootLibrary>) -> Result<()> {
        match self {
            Self::Dna(id) => match libraries.get(id.as_id()) {
                Some(RootLibrary::IlluminaDna(_)) => Ok(()),
                Some(_) => bail!(
                    "Illumina DNA library reference '{}' resolves to a different library type",
                    id
                ),
                None => bail!("Assay references unknown Illumina DNA Library '{id}'"),
            },
            Self::Cdna(id) => match libraries.get(id.as_id()) {
                Some(RootLibrary::IlluminaCdna(_)) => Ok(()),
                Some(_) => bail!(
                    "Illumina cDNA library reference '{}' resolves to a different library type",
                    id
                ),
                None => bail!("Assay references unknown Illumina cDNA Library '{id}'"),
            },
        }
    }
}

/// Standard single-end Illumina sequencing of a compatible library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SingleEndSequencing {
    core: SequencingAssayCore<SingleEndSequencingId>,
}

impl SingleEndSequencing {
    /// Creates a single-end Illumina sequencing assay.
    pub fn new(
        id: SingleEndSequencingId,
        library: SequencingInput,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: SequencingAssayCore::new(id, library, meta, description)?,
        })
    }

    fn from_parts(
        id: SingleEndSequencingId,
        library: SequencingInput,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: SequencingAssayCore {
                id,
                library,
                meta,
                description,
            },
        }
    }

    /// Returns this assay's identifier.
    pub fn id(&self) -> &SingleEndSequencingId {
        &self.core.id
    }

    /// Returns this assay's typed library input.
    pub fn library(&self) -> &SequencingInput {
        &self.core.library
    }

    pub(crate) fn validate_references(&self, libraries: &BTreeMap<Id, RootLibrary>) -> Result<()> {
        self.library().validate_references(libraries)
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

/// Standard paired-end Illumina sequencing of a compatible library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedEndSequencing {
    core: SequencingAssayCore<PairedEndSequencingId>,
}

impl PairedEndSequencing {
    /// Creates a paired-end Illumina sequencing assay.
    pub fn new(
        id: PairedEndSequencingId,
        library: SequencingInput,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: SequencingAssayCore::new(id, library, meta, description)?,
        })
    }

    fn from_parts(
        id: PairedEndSequencingId,
        library: SequencingInput,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: SequencingAssayCore {
                id,
                library,
                meta,
                description,
            },
        }
    }

    /// Returns this assay's identifier.
    pub fn id(&self) -> &PairedEndSequencingId {
        &self.core.id
    }

    /// Returns this assay's typed library input.
    pub fn library(&self) -> &SequencingInput {
        &self.core.library
    }

    pub(crate) fn validate_references(&self, libraries: &BTreeMap<Id, RootLibrary>) -> Result<()> {
        self.library().validate_references(libraries)
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
struct SequencingAssayCore<I> {
    id: I,
    library: SequencingInput,
    meta: Meta,
    description: Option<String>,
}

impl<I> SequencingAssayCore<I> {
    fn new(
        id: I,
        library: SequencingInput,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            library,
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
            library: self.library().id(),
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
            library: self.library().id(),
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedSingleEndSequencing<'a> {
    id: &'a SingleEndSequencingId,
    library: &'a Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Serialize)]
struct SerializedPairedEndSequencing<'a> {
    id: &'a PairedEndSequencingId,
    library: &'a Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedSingleEndSequencing {
    id: SingleEndSequencingId,
    library: Id,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedPairedEndSequencing {
    id: PairedEndSequencingId,
    library: Id,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

impl UnresolvedSingleEndSequencing {
    pub(crate) fn resolve(
        self,
        libraries: &BTreeMap<Id, RootLibrary>,
    ) -> Result<SingleEndSequencing> {
        let library = SequencingInput::resolve(self.library, libraries)?;
        Ok(SingleEndSequencing::from_parts(
            self.id,
            library,
            self.meta,
            self.description,
        ))
    }
}

impl UnresolvedPairedEndSequencing {
    pub(crate) fn resolve(
        self,
        libraries: &BTreeMap<Id, RootLibrary>,
    ) -> Result<PairedEndSequencing> {
        let library = SequencingInput::resolve(self.library, libraries)?;
        Ok(PairedEndSequencing::from_parts(
            self.id,
            library,
            self.meta,
            self.description,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{DnaLibraryId, SequencingInput, SingleEndSequencing, SingleEndSequencingId};
    use crate::provenance::Assay;

    #[test]
    fn serializes_the_library_reference_without_repeating_its_type() {
        let assay = SingleEndSequencing::new(
            SingleEndSequencingId::new("ASY1").unwrap(),
            SequencingInput::Dna(DnaLibraryId::new("LIB1").unwrap()),
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&Assay::IlluminaSingleEndSequencing(assay)).unwrap(),
            r#"{"type":"IlluminaSingleEndSequencing","id":"ASY1","library":"LIB1"}"#
        );
    }
}
