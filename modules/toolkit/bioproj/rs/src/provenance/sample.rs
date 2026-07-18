use crate::primitives::{NonEmptyIdSet, define_entity_id};
use crate::{Meta, MetaVal};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::source::SourceId;

define_entity_id!(
    SampleId,
    "The identifier of a [`crate::provenance::Sample`]."
);

/// Physical material extracted from one or more biological sources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    id: SampleId,
    sources: NonEmptyIdSet<SourceId>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl Sample {
    /// Creates a sample with one or more parent sources.
    pub fn new(
        id: SampleId,
        sources: impl IntoIterator<Item = SourceId>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            sources: NonEmptyIdSet::new("Sample::sources", sources)?,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }

    /// Returns this sample's identifier.
    pub fn id(&self) -> &SampleId {
        &self.id
    }

    /// Returns the IDs of this sample's parent sources.
    pub fn sources(&self) -> &BTreeSet<SourceId> {
        self.sources.as_set()
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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
                [("kind", "tissue")],
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
                [("kind", "tissue")],
                None::<String>,
            )
            .is_err()
        );
    }
}
