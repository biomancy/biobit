//! Libraries exposing the standard P5/P7 sequencing interface.

use crate::Strandedness;
use crate::primitives::define_entity_id;
use crate::provenance::SampleId;
use crate::{Meta, NonEmpty, OrderedSet, Text};

define_entity_id!(
    LibraryId,
    "The identifier of a [`crate::provenance::library::p5p7::Library`]."
);

/// The biological-molecule path used to construct a P5/P7 library.
///
/// This deliberately compresses upstream preparation rather than attempting
/// to model it as a protocol graph. Details such as extraction, enrichment,
/// kits, and UMIs remain library metadata until they have a stable schema.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum Input {
    /// The library was constructed from DNA.
    Dna,
    /// The library was constructed from RNA.
    Rna {
        /// The resulting library's relationship to the source RNA strand.
        /// `None` means that strand specificity was not reported.
        strandedness: Option<Strandedness>,
    },
}

/// A sequencing-ready library exposing a standard P5/P7 interface.
///
/// The library has one preparation path shared by all contributing samples.
/// Samples prepared through different DNA- or RNA-derived paths must be
/// represented as separate libraries.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Library {
    id: LibraryId,
    samples: NonEmpty<OrderedSet<SampleId>>,
    input: Input,
    meta: Meta,
    description: Option<Text>,
}

impl Library {
    /// Creates a P5/P7 library from one or more samples and one input path.
    pub fn new(
        id: LibraryId,
        samples: NonEmpty<OrderedSet<SampleId>>,
        input: Input,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            samples,
            input,
            meta,
            description,
        }
    }

    /// Returns this library's identifier.
    pub fn id(&self) -> &LibraryId {
        &self.id
    }

    /// Returns the IDs of all samples that contributed material.
    pub fn samples(&self) -> &NonEmpty<OrderedSet<SampleId>> {
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
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }
}
