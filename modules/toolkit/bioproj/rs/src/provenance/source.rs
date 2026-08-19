use crate::primitives::define_entity_id;
use crate::{Meta, NonEmpty};
use eyre::Result;
use serde::{Deserialize, Serialize};

define_entity_id!(
    SourceId,
    "The identifier of a [`crate::provenance::Source`]."
);

/// A stable biological origin, such as a donor, cell line, or pathogen.
///
/// A source identifies one biological entity and carries facts shared by all
/// material derived from it, such as organism, genotype, or prior treatment
/// history. Conditions specific to collected material belong to its
/// [`Sample`](crate::provenance::Sample); mixtures are represented by a sample
/// that references multiple sources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    id: SourceId,
    organism: NonEmpty<String>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl Source {
    /// Creates a source with its organism.
    pub fn new(
        id: SourceId,
        organism: impl Into<String>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            organism: NonEmpty::new(organism.into())?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this source's identifier.
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Returns the source organism.
    pub fn organism(&self) -> &NonEmpty<String> {
        &self.organism
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
    use super::{Source, SourceId};
    use crate::{Meta, MetaVal, NonEmpty};
    use std::collections::BTreeMap;

    #[test]
    fn stores_organism_and_boolean_metadata() {
        let source = Source::new(
            SourceId::new("SRC1").unwrap(),
            "Homo sapiens",
            Meta::from(BTreeMap::from([(
                NonEmpty::new("is_control".to_owned()).unwrap(),
                MetaVal::from(true),
            )])),
            None::<String>,
        )
        .unwrap();

        assert_eq!(source.organism().as_ref(), "Homo sapiens");
        assert_eq!(source.meta().get("is_control"), Some(&MetaVal::Bool(true)));
    }

    #[test]
    fn requires_an_organism() {
        assert!(
            Source::new(
                SourceId::new("SRC1").unwrap(),
                "",
                Default::default(),
                None::<String>,
            )
            .is_err()
        );
    }

    #[test]
    fn deserialization_requires_a_non_empty_organism() {
        assert!(serde_json::from_str::<Source>(r#"{"id":"SRC1","organism":""}"#).is_err());
    }

    #[test]
    fn deserialization_rejects_an_empty_description() {
        assert!(
            serde_json::from_str::<Source>(
                r#"{"id":"SRC1","organism":"Homo sapiens","description":""}"#
            )
            .is_err()
        );
    }

    #[test]
    fn typed_id_preserves_the_common_identifier() {
        let id = SourceId::new("SRC1").unwrap();
        assert_eq!(id.as_untyped().as_str(), "SRC1");
    }
}
