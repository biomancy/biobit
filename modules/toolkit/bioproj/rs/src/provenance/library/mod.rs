//! Concrete, acquisition-ready library preparations.

pub mod p5p7;
pub mod strandedness;

use crate::{Meta, NonEmpty, UntypedId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::SampleId;

/// A concrete, acquisition-ready library preparation.
///
/// Each variant describes the stable physical interface presented to
/// compatible acquisitions. Its concrete value records the input material and
/// preparation details.
///
/// For example, a [`Library::P5P7`] library is prepared by adding P5 and P7
/// adapter sequences to the input material and it could be then sequenced
/// by a number of different sequencing platforms.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Library {
    /// A library exposing the standard P5/P7 sequencing adapters.
    P5P7(p5p7::Library),
}

impl Library {
    /// Returns the untyped workspace-local identifier for this library.
    pub fn untyped_id(&self) -> &UntypedId {
        match self {
            Self::P5P7(library) => library.id().as_untyped(),
        }
    }

    /// Returns the IDs of this library's parent samples.
    pub fn samples(&self) -> &NonEmpty<BTreeSet<SampleId>> {
        match self {
            Self::P5P7(library) => library.samples(),
        }
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        match self {
            Self::P5P7(library) => library.meta(),
        }
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        match self {
            Self::P5P7(library) => library.description(),
        }
    }
}

impl From<p5p7::Library> for Library {
    fn from(library: p5p7::Library) -> Self {
        Self::P5P7(library)
    }
}

#[cfg(test)]
mod tests {
    use super::Library;
    use crate::provenance::SampleId;
    use crate::provenance::library::p5p7::{Input, Library as P5P7, LibraryId};
    use crate::provenance::library::strandedness::Strandedness;

    #[test]
    fn serializes_flat_tagged_library_variants() {
        let library = Library::P5P7(
            P5P7::new(
                LibraryId::new("LIB1").unwrap(),
                [SampleId::new("SMP1").unwrap()],
                Input::FromRna {
                    strandedness: Strandedness::Forward,
                },
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        );

        assert_eq!(
            serde_json::to_string(&library).unwrap(),
            r#"{"type":"P5P7","id":"LIB1","samples":["SMP1"],"input":{"type":"FromRna","strandedness":"Forward"}}"#
        );
    }

    #[test]
    fn exposes_fields_common_to_all_library_variants() {
        let library: Library = P5P7::new(
            LibraryId::new("LIB1").unwrap(),
            [SampleId::new("SMP1").unwrap()],
            Input::FromDna,
            Default::default(),
            Some("P5/P7 library"),
        )
        .unwrap()
        .into();

        assert!(library.meta().is_empty());
        assert_eq!(
            library
                .description()
                .map(|description| description.as_str()),
            Some("P5/P7 library")
        );
    }
}
