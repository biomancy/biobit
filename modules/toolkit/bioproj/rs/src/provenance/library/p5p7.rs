//! Libraries exposing the standard P5/P7 sequencing interface.

use super::strandedness::Strandedness;
use crate::primitives::define_entity_id;
use crate::provenance::SampleId;
use crate::{Meta, NonEmpty};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

define_entity_id!(
    LibraryId,
    "The identifier of a [`crate::provenance::library::p5p7::Library`]."
);

/// The biological-molecule path used to construct a P5/P7 library.
///
/// This deliberately compresses upstream preparation rather than attempting
/// to model it as a protocol graph. Details such as extraction, enrichment,
/// kits, and UMIs remain library metadata until they have a stable schema.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Input {
    /// The library was constructed from DNA.
    FromDna,
    /// The library was constructed from RNA.
    FromRna {
        /// The resulting library's relationship to the source RNA strand.
        strandedness: Strandedness,
    },
}

/// A sequencing-ready library exposing a standard P5/P7 interface.
///
/// The library has one preparation path shared by all contributing samples.
/// Samples prepared through different DNA- or RNA-derived paths must be
/// represented as separate libraries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    id: LibraryId,
    samples: NonEmpty<BTreeSet<SampleId>>,
    input: Input,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl Library {
    /// Creates a P5/P7 library from one or more samples and one input path.
    pub fn new(
        id: LibraryId,
        samples: impl IntoIterator<Item = SampleId>,
        input: Input,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            samples: NonEmpty::try_from_iter(samples)?,
            input,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this library's identifier.
    pub fn id(&self) -> &LibraryId {
        &self.id
    }

    /// Returns the IDs of all samples that contributed material.
    pub fn samples(&self) -> &NonEmpty<BTreeSet<SampleId>> {
        &self.samples
    }

    /// Returns the single biological-molecule path used for construction.
    pub fn input(&self) -> &Input {
        &self.input
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

#[cfg(test)]
mod tests {
    use super::{Input, Library, LibraryId};
    use crate::provenance::SampleId;
    use crate::provenance::library::strandedness::Strandedness;

    fn sample_id(id: &str) -> SampleId {
        SampleId::new(id).unwrap()
    }

    #[test]
    fn represents_exactly_one_dna_or_rna_input_path() {
        let dna = Library::new(
            LibraryId::new("LIB_DNA").unwrap(),
            [sample_id("SMP1")],
            Input::FromDna,
            Default::default(),
            None::<String>,
        )
        .unwrap();
        let rna = Library::new(
            LibraryId::new("LIB_RNA").unwrap(),
            [sample_id("SMP1")],
            Input::FromRna {
                strandedness: Strandedness::Unknown,
            },
            Default::default(),
            None::<String>,
        )
        .unwrap();

        assert!(matches!(dna.input(), Input::FromDna));
        assert_eq!(
            rna.input(),
            &Input::FromRna {
                strandedness: Strandedness::Unknown,
            }
        );
    }

    #[test]
    fn round_trips_the_tagged_input() {
        let library = Library::new(
            LibraryId::new("LIB1").unwrap(),
            [sample_id("SMP1")],
            Input::FromRna {
                strandedness: Strandedness::Forward,
            },
            Default::default(),
            None::<String>,
        )
        .unwrap();

        let json = serde_json::to_string(&library).unwrap();
        assert_eq!(serde_json::from_str::<Library>(&json).unwrap(), library);
    }

    #[test]
    fn rna_input_requires_explicit_strandedness() {
        assert!(
            serde_json::from_str::<Library>(
                r#"{
                    "id": "LIB1",
                    "samples": ["SMP1"],
                    "input": {"type": "FromRna"}
                }"#,
            )
            .is_err()
        );
    }
}
