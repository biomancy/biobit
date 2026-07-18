use crate::primitives::define_entity_id;
use crate::validation;
use crate::{Meta, MetaVal};
use eyre::Result;
use serde::{Deserialize, Serialize};

define_entity_id!(
    SourceId,
    "The identifier of a [`crate::provenance::Source`]."
);

/// A biological root, such as a donor, cell line, or pathogen.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    id: SourceId,
    #[serde(deserialize_with = "validation::deserialize_non_empty_string")]
    organism: String,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl Source {
    /// Creates a source with its organism.
    pub fn new(
        id: SourceId,
        organism: impl Into<String>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            organism: validation::non_empty_string("Source::organism", organism)?,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }

    /// Returns this source's identifier.
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Returns the source organism.
    pub fn organism(&self) -> &str {
        &self.organism
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
    use super::{Source, SourceId};
    use crate::MetaVal;

    #[test]
    fn stores_organism_and_boolean_metadata() {
        let source = Source::new(
            SourceId::new("SRC1").unwrap(),
            "Homo sapiens",
            [("is_control", true)],
            None::<String>,
        )
        .unwrap();

        assert_eq!(source.organism(), "Homo sapiens");
        assert_eq!(source.meta().get("is_control"), Some(&MetaVal::Bool(true)));
    }

    #[test]
    fn requires_an_organism() {
        assert!(
            Source::new(
                SourceId::new("SRC1").unwrap(),
                "",
                [("kind", "donor")],
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
    fn typed_id_preserves_the_common_identifier() {
        let id = SourceId::new("SRC1").unwrap();
        assert_eq!(id.as_id().as_str(), "SRC1");
    }
}
