use crate::primitives::define_entity_id;
use crate::provenance::{AcquisitionId, Provenance};
use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

define_entity_id!(
    DesignUnitId,
    "The identifier of a [`crate::design::DesignUnit`]."
);

/// A named, non-empty collection of acquisitions pooled for computation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesignUnit {
    id: DesignUnitId,
    acquisitions: NonEmpty<BTreeSet<AcquisitionId>>,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl DesignUnit {
    /// Creates a design unit from one or more acquisitions.
    pub fn new(
        id: DesignUnitId,
        acquisitions: impl IntoIterator<Item = AcquisitionId>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            acquisitions: NonEmpty::try_from_iter(acquisitions)?,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }

    fn from_parts(
        id: DesignUnitId,
        acquisitions: NonEmpty<BTreeSet<AcquisitionId>>,
        meta: Meta,
        description: Option<NonEmpty<String>>,
    ) -> Self {
        Self {
            id,
            acquisitions,
            meta,
            description,
        }
    }

    /// Returns this design unit's identifier.
    pub fn id(&self) -> &DesignUnitId {
        &self.id
    }

    /// Returns the acquisitions logically pooled in this unit.
    pub fn acquisitions(&self) -> &NonEmpty<BTreeSet<AcquisitionId>> {
        &self.acquisitions
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedDesignUnit {
    id: DesignUnitId,
    acquisitions: NonEmpty<BTreeSet<UntypedId>>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<NonEmpty<String>>,
}

impl UnresolvedDesignUnit {
    pub(crate) fn resolve(self, provenance: &Provenance) -> Result<DesignUnit> {
        let acquisitions = self
            .acquisitions
            .into_inner()
            .into_iter()
            .map(|id| {
                provenance
                    .acquisition(&id)
                    .map(|acquisition| acquisition.id().to_owned())
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "DesignUnit '{}' references unknown Acquisition '{id}'",
                            self.id
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(DesignUnit::from_parts(
            self.id,
            NonEmpty::try_from_iter(acquisitions)?,
            self.meta,
            self.description,
        ))
    }
}
