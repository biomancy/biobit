use super::p5p7;
use crate::NonEmpty;
use crate::primitives::define_entity_family;
use crate::provenance::SampleId;
use std::collections::BTreeSet;

define_entity_family! {
    /// A concrete, acquisition-ready library preparation.
    ///
    /// Each variant describes the stable physical interface presented to
    /// compatible acquisitions. Its concrete value records the input material and
    /// preparation details.
    ///
    /// For example, a [`Library::P5P7`] library is prepared by adding P5 and P7
    /// adapter sequences to the input material and can then be sequenced by a
    /// number of different sequencing platforms.
    pub family Library {
        kind: LibraryKind,
        id: LibraryId,
        id_ref: LibraryIdRef,
        kind_doc: "The concrete type of a library or library identifier.",
        variants: {
            /// A library exposing the standard P5/P7 sequencing adapters.
            P5P7(p5p7::Library, p5p7::LibraryId),
        }
    }
}

impl Library {
    /// Returns the IDs of this library's parent samples.
    pub fn samples(&self) -> &NonEmpty<BTreeSet<SampleId>> {
        match self {
            Self::P5P7(library) => library.samples(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Library, LibraryId as AnyLibraryId, LibraryKind};
    use crate::provenance::SampleId;
    use crate::provenance::library::p5p7::{Input, Library as P5P7, LibraryId};
    use crate::provenance::library::strandedness::Strandedness;
    use kinded::Kinded as _;

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

    #[test]
    fn distinguishes_kind_from_strict_identity_matching() {
        let id = LibraryId::new("LIB1").unwrap();
        let library: Library = P5P7::new(
            id.clone(),
            [SampleId::new("SMP1").unwrap()],
            Input::FromDna,
            Default::default(),
            None::<String>,
        )
        .unwrap()
        .into();

        let matching = AnyLibraryId::from(id);
        let same_kind = AnyLibraryId::from(LibraryId::new("LIB2").unwrap());

        assert_eq!(matching.kind(), LibraryKind::P5P7);
        assert_eq!(matching.kind(), same_kind.kind());
        assert!(matching.matches(&library));
        assert!(!same_kind.matches(&library));
    }
}
