use crate::primitives::define_entity_id;
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text};

use super::{Provenance, source::SourceId};

define_entity_id!(
    SampleId,
    "The identifier of a [`crate::provenance::Sample`]."
);

/// A collected portion of biological material from one or more sources.
///
/// A sample captures the material's condition at collection, so details such
/// as tissue, treatment, or time point belong in its `meta`. Multiple samples
/// may come from the same source, while a mixture references its known origins.
/// These references do not assert exhaustive identification of its composition.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Sample {
    id: SampleId,
    sources: NonEmpty<OrderedSet<SourceId>>,
    meta: Meta,
    description: Option<Text>,
}

impl Sample {
    /// Creates a sample with every source that contributed material.
    pub fn new(
        id: SampleId,
        sources: NonEmpty<OrderedSet<SourceId>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            sources,
            meta,
            description,
        }
    }

    /// Returns this sample's identifier.
    pub fn id(&self) -> &SampleId {
        &self.id
    }

    /// Returns the IDs of all sources that contributed material.
    pub fn sources(&self) -> &NonEmpty<OrderedSet<SourceId>> {
        &self.sources
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }

    /// Validates every Sample-to-Source edge in a provenance graph.
    pub(crate) fn validate_against(provenance: &Provenance) -> Result<()> {
        for sample in provenance.samples() {
            for source_id in sample.sources() {
                if provenance.source(source_id.as_untyped()).is_none() {
                    return Err(Error::missing(
                        "Sample",
                        sample.id().as_untyped(),
                        "sources",
                        "Source",
                        source_id.as_untyped(),
                    ));
                }
            }
        }
        Ok(())
    }
}
