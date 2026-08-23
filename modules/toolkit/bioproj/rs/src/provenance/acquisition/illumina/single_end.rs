use super::validate;
use crate::primitives::{define_entity_id, impl_kind};
use crate::provenance::acquisition::AcquisitionKind;
use crate::provenance::library::Library;
use crate::provenance::library::p5p7;
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

define_entity_id!(
    SingleEndSequencingId,
    "The identifier of a [`bioproj::provenance::acquisition::illumina::SingleEndSequencing`]."
);

/// One logical single-end Illumina sequencing acquisition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleEndSequencing {
    id: SingleEndSequencingId,
    libraries: NonEmpty<BTreeSet<p5p7::LibraryId>>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl_kind!(
    AcquisitionKind,
    IlluminaSingleEndSequencing => SingleEndSequencing, SingleEndSequencingId,
);

impl SingleEndSequencing {
    /// Creates a single-end acquisition from one or more intentionally pooled libraries.
    pub fn new(
        id: SingleEndSequencingId,
        libraries: impl IntoIterator<Item = p5p7::LibraryId>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            libraries: NonEmpty::try_from_iter(libraries)?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    /// Returns this acquisition's identifier.
    pub fn id(&self) -> &SingleEndSequencingId {
        &self.id
    }

    /// Returns the libraries intentionally pooled into this acquisition.
    pub fn libraries(&self) -> &NonEmpty<BTreeSet<p5p7::LibraryId>> {
        &self.libraries
    }

    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        validate::validate(&self.libraries, libraries)
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
    use crate::provenance::Acquisition;
    use crate::provenance::library::p5p7;

    #[test]
    fn requires_and_serializes_one_or_more_libraries() {
        assert!(
            SingleEndSequencing::new(
                SingleEndSequencingId::new("ACQ1").unwrap(),
                Vec::<p5p7::LibraryId>::new(),
                Default::default(),
                None::<String>,
            )
            .is_err()
        );

        let acquisition = Acquisition::IlluminaSingleEndSequencing(
            SingleEndSequencing::new(
                SingleEndSequencingId::new("ACQ1").unwrap(),
                [
                    p5p7::LibraryId::new("LIB1").unwrap(),
                    p5p7::LibraryId::new("LIB2").unwrap(),
                ],
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        );

        assert_eq!(
            serde_json::to_string(&acquisition).unwrap(),
            r#"{"type":"IlluminaSingleEndSequencing","id":"ACQ1","libraries":["LIB1","LIB2"]}"#
        );
    }
}
