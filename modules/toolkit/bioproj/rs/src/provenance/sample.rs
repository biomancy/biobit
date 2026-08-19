use crate::primitives::define_entity_id;
use crate::{Meta, NonEmpty};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::source::SourceId;

define_entity_id!(
    SampleId,
    "The identifier of a [`crate::provenance::Sample`]."
);

/// A collected portion of biological material from one or more sources.
///
/// A sample captures the material's condition at collection, so details such
/// as tissue, treatment, or time point belong in its `meta`. Multiple samples
/// may come from the same source, while a mixture references every source that
/// contributed material.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    id: SampleId,
    sources: NonEmpty<BTreeSet<SourceId>>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl Sample {
    /// Creates a sample with every source that contributed material.
    pub fn new(
        id: SampleId,
        sources: impl IntoIterator<Item = SourceId>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            sources: NonEmpty::try_from_iter(sources)?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this sample's identifier.
    pub fn id(&self) -> &SampleId {
        &self.id
    }

    /// Returns the IDs of all sources that contributed material.
    pub fn sources(&self) -> &NonEmpty<BTreeSet<SourceId>> {
        &self.sources
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
    use super::{Sample, SampleId};
    use crate::provenance::source::SourceId;

    #[test]
    fn requires_distinct_parent_sources() {
        assert!(
            Sample::new(
                SampleId::new("SMP1").unwrap(),
                Vec::<SourceId>::new(),
                Default::default(),
                None::<String>,
            )
            .is_err()
        );
        assert!(
            Sample::new(
                SampleId::new("SMP1").unwrap(),
                [
                    SourceId::new("SRC1").unwrap(),
                    SourceId::new("SRC1").unwrap()
                ],
                Default::default(),
                None::<String>,
            )
            .is_err()
        );
    }
}
