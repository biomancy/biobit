use crate::{Error, Result};

use super::p5p7;
use crate::primitives::define_entity_family;
use crate::provenance::{Provenance, SampleId};
use crate::{NonEmpty, OrderedSet};

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
    pub fn samples(&self) -> &NonEmpty<OrderedSet<SampleId>> {
        match self {
            Self::P5P7(library) => library.samples(),
        }
    }

    /// Validates every Library-to-Sample edge in a provenance graph.
    pub(crate) fn validate_against(provenance: &Provenance) -> Result<()> {
        for library in provenance.libraries() {
            for sample_id in library.samples() {
                if provenance.sample(sample_id.as_untyped()).is_none() {
                    return Err(Error::missing(
                        "Library",
                        library.id().as_untyped(),
                        "samples",
                        "Sample",
                        sample_id.as_untyped(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Library, LibraryId as AnyLibraryId, LibraryKind};
    use crate::provenance::SampleId;
    use crate::provenance::library::p5p7::{Input, Library as P5P7, LibraryId};
    use crate::{NonEmpty, Text};
    use kinded::Kinded as _;

    #[test]
    fn exposes_fields_common_to_all_library_variants() {
        let library: Library = P5P7::new(
            LibraryId::new("LIB1").unwrap(),
            NonEmpty::try_from_iter([SampleId::new("SMP1").unwrap()]).unwrap(),
            Input::Dna,
            Default::default(),
            Some(Text::new("P5/P7 library").unwrap()),
        )
        .into();

        assert!(library.meta().is_empty());
        let description: Option<&Text> = library.description();
        assert_eq!(description.unwrap().as_str(), "P5/P7 library");
    }

    #[test]
    fn family_ids_compare_with_borrowed_entity_ids() {
        let id = LibraryId::new("LIB1").unwrap();
        let library: Library = P5P7::new(
            id.clone(),
            NonEmpty::try_from_iter([SampleId::new("SMP1").unwrap()]).unwrap(),
            Input::Dna,
            Default::default(),
            None,
        )
        .into();

        let matching = AnyLibraryId::from(id);
        let same_kind = AnyLibraryId::from(LibraryId::new("LIB2").unwrap());

        assert_eq!(matching.kind(), LibraryKind::P5P7);
        assert_eq!(matching.kind(), same_kind.kind());
        assert_eq!(matching, library.id());
        assert_ne!(same_kind, library.id());
    }
}
