use super::validate;
use crate::primitives::define_entity_id;
use crate::provenance::library::Library;
use crate::provenance::library::p5p7::LibraryId;
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

define_entity_id!(
    SingleEndSequencingId,
    "The identifier of a [`crate::provenance::assay::illumina::SingleEndSequencing`]."
);

/// Standard single-end Illumina sequencing of a compatible library.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleEndSequencing {
    id: SingleEndSequencingId,
    library: LibraryId,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl SingleEndSequencing {
    /// Creates a single-end Illumina sequencing assay.
    pub fn new(
        id: SingleEndSequencingId,
        library: LibraryId,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            library,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this assay's identifier.
    pub fn id(&self) -> &SingleEndSequencingId {
        &self.id
    }

    /// Returns this assay's typed library input.
    pub fn library(&self) -> &LibraryId {
        &self.library
    }

    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        validate::validate(self.library(), libraries)
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
    use super::{SingleEndSequencing, SingleEndSequencingId};
    use crate::provenance::Assay;
    use crate::provenance::library::p5p7::LibraryId;

    #[test]
    fn serializes_the_library_reference_without_repeating_its_type() {
        let assay = SingleEndSequencing::new(
            SingleEndSequencingId::new("ASY1").unwrap(),
            LibraryId::new("LIB1").unwrap(),
            Default::default(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&Assay::IlluminaSingleEndSequencing(assay)).unwrap(),
            r#"{"type":"IlluminaSingleEndSequencing","id":"ASY1","library":"LIB1"}"#
        );
    }

    #[test]
    fn round_trips_through_the_root_assay_enum() {
        let library_id = LibraryId::new("LIB1").unwrap();
        let assay = Assay::IlluminaSingleEndSequencing(
            SingleEndSequencing::new(
                SingleEndSequencingId::new("ASY1").unwrap(),
                library_id.clone(),
                Default::default(),
                Some("single-end acquisition"),
            )
            .unwrap(),
        );

        assert_eq!(assay.library(), library_id.as_untyped());
        assert!(assay.meta().is_empty());
        assert_eq!(
            assay.description().map(|description| description.as_str()),
            Some("single-end acquisition")
        );

        let json = serde_json::to_string(&assay).unwrap();
        assert_eq!(serde_json::from_str::<Assay>(&json).unwrap(), assay);
    }
}
