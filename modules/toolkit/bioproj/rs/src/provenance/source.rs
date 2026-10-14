use crate::primitives::define_entity_id;
use crate::{Meta, Text};

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
/// that references its known contributing sources. Unknown biological composition
/// does not require inventing Source identities. Organism may be unreported.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Source {
    id: SourceId,
    organism: Option<Text>,
    meta: Meta,
    description: Option<Text>,
}

impl Source {
    /// Creates a source with an optional organism.
    pub fn new(
        id: SourceId,
        organism: Option<Text>,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            organism,
            meta,
            description,
        }
    }

    /// Returns this source's identifier.
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Returns the reported organism, when known.
    pub fn organism(&self) -> Option<&Text> {
        self.organism.as_ref()
    }

    /// Returns auxiliary metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{Source, SourceId};
    use crate::{Meta, MetaVal, Text};
    use std::collections::BTreeMap;

    #[test]
    fn stores_organism_and_boolean_metadata() {
        let source = Source::new(
            SourceId::new("SRC1").unwrap(),
            Some(Text::new("Homo sapiens").unwrap()),
            Meta::from(BTreeMap::from([(
                Text::new("is_control").unwrap(),
                MetaVal::from(true),
            )])),
            None,
        );

        assert_eq!(source.organism().unwrap().as_str(), "Homo sapiens");
        assert_eq!(source.meta().get("is_control"), Some(&MetaVal::Bool(true)));
    }

    #[test]
    fn permits_an_unknown_or_unreported_organism() {
        let source = Source::new(SourceId::new("SRC1").unwrap(), None, Meta::default(), None);
        assert_eq!(source.organism(), None);
    }

    #[test]
    fn typed_id_preserves_the_common_identifier() {
        let id = SourceId::new("SRC1").unwrap();
        assert_eq!(id.as_untyped().as_str(), "SRC1");
    }
}
