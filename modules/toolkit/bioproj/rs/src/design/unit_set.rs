use super::DesignUnit;
use super::core::DesignCore;
use super::{DesignId, DesignUnitId};
use crate::primitives::NonEmptyIdSet;
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

/// An unordered collection of design units for a joint analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitSet {
    core: DesignCore,
    units: NonEmptyIdSet<DesignUnitId>,
}

impl UnitSet {
    /// Creates a non-empty unordered collection of design units.
    pub fn new(
        id: DesignId,
        units: impl IntoIterator<Item = DesignUnitId>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            core: DesignCore::new(id, meta, description)?,
            units: NonEmptyIdSet::new("UnitSet::units", units)?,
        })
    }

    fn from_parts(
        id: DesignId,
        units: NonEmptyIdSet<DesignUnitId>,
        meta: Meta,
        description: Option<String>,
    ) -> Self {
        Self {
            core: DesignCore {
                id,
                meta,
                description,
            },
            units,
        }
    }

    /// Returns this design's identifier.
    pub fn id(&self) -> &DesignId {
        &self.core.id
    }

    /// Returns the unordered design units.
    pub fn units(&self) -> &BTreeSet<DesignUnitId> {
        self.units.as_set()
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.core.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&str> {
        self.core.description.as_deref()
    }

    pub(crate) fn validate_references(
        &self,
        units: &BTreeMap<DesignUnitId, DesignUnit>,
    ) -> Result<()> {
        for unit_id in self.units() {
            if !units.contains_key(unit_id) {
                bail!(
                    "UnitSet Design '{}' references unknown DesignUnit '{unit_id}'",
                    self.id()
                );
            }
        }
        Ok(())
    }
}

impl AsRef<Id> for UnitSet {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

impl Serialize for UnitSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedUnitSet {
            id: self.id(),
            units: &self.units,
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedUnitSet<'a> {
    id: &'a DesignId,
    units: &'a NonEmptyIdSet<DesignUnitId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

impl<'de> Deserialize<'de> for UnitSet {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let design = DeserializedUnitSet::deserialize(deserializer)?;
        Ok(Self::from_parts(
            design.id,
            design.units,
            design.meta,
            design.description,
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedUnitSet {
    id: DesignId,
    units: NonEmptyIdSet<DesignUnitId>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::UnitSet;
    use crate::design::{DesignId, DesignUnitId};

    #[test]
    fn requires_distinct_units() {
        let unit = DesignUnitId::new("UNIT1").unwrap();
        assert!(
            UnitSet::new(
                DesignId::new("DES1").unwrap(),
                [unit.clone(), unit],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }
}
