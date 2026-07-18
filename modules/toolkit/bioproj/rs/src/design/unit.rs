use crate::primitives::{NonEmptyIdSet, define_entity_id};
use crate::{Id, Meta, MetaVal};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

define_entity_id!(
    DesignUnitId,
    "The identifier of a [`crate::design::DesignUnit`]."
);

/// A named, non-empty collection of assays logically pooled for computation.
///
/// Assay references use the common [`Id`] type deliberately: design units can
/// pool assays of any current or future concrete assay type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignUnit {
    id: DesignUnitId,
    assays: NonEmptyIdSet<Id>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl DesignUnit {
    /// Creates a design unit from one or more assays.
    pub fn new(
        id: DesignUnitId,
        assays: impl IntoIterator<Item = Id>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            assays: NonEmptyIdSet::new("DesignUnit::assays", assays)?,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }

    /// Returns this design unit's identifier.
    pub fn id(&self) -> &DesignUnitId {
        &self.id
    }

    /// Returns the assays logically pooled in this unit.
    pub fn assays(&self) -> &BTreeSet<Id> {
        self.assays.as_set()
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

impl AsRef<Id> for DesignUnit {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

#[cfg(test)]
mod tests {
    use super::{DesignUnit, DesignUnitId};
    use crate::Id;

    #[test]
    fn requires_distinct_assays() {
        assert!(
            DesignUnit::new(
                DesignUnitId::new("UNIT1").unwrap(),
                Vec::<Id>::new(),
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );

        let assay = Id::new("ASY1").unwrap();
        assert!(
            DesignUnit::new(
                DesignUnitId::new("UNIT1").unwrap(),
                [assay.clone(), assay],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }
}
