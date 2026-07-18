use super::DesignUnit;
use super::core::DesignCore;
use super::{DesignId, DesignUnitId};
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail, ensure};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;

/// A direct contrast between one control and one treatment design unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwoUnits {
    core: DesignCore,
    control: DesignUnitId,
    treatment: DesignUnitId,
}

impl TwoUnits {
    /// Creates a direct contrast between two distinct design units.
    pub fn new(
        id: DesignId,
        control: DesignUnitId,
        treatment: DesignUnitId,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        validate_distinct_units(&control, &treatment)?;
        Ok(Self {
            core: DesignCore::new(id, meta, description)?,
            control,
            treatment,
        })
    }

    fn from_parts(
        id: DesignId,
        control: DesignUnitId,
        treatment: DesignUnitId,
        meta: Meta,
        description: Option<String>,
    ) -> Result<Self> {
        validate_distinct_units(&control, &treatment)?;
        Ok(Self {
            core: DesignCore {
                id,
                meta,
                description,
            },
            control,
            treatment,
        })
    }

    /// Returns this design's identifier.
    pub fn id(&self) -> &DesignId {
        &self.core.id
    }

    /// Returns the control design unit.
    pub fn control(&self) -> &DesignUnitId {
        &self.control
    }

    /// Returns the treatment design unit.
    pub fn treatment(&self) -> &DesignUnitId {
        &self.treatment
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
        for unit_id in [self.control(), self.treatment()] {
            if !units.contains_key(unit_id) {
                bail!(
                    "TwoUnits Design '{}' references unknown DesignUnit '{unit_id}'",
                    self.id()
                );
            }
        }
        Ok(())
    }
}

impl AsRef<Id> for TwoUnits {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

fn validate_distinct_units(control: &DesignUnitId, treatment: &DesignUnitId) -> Result<()> {
    ensure!(
        control != treatment,
        "TwoUnits::control and TwoUnits::treatment must be distinct"
    );
    Ok(())
}

impl Serialize for TwoUnits {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedTwoUnits {
            id: self.id(),
            control: self.control(),
            treatment: self.treatment(),
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedTwoUnits<'a> {
    id: &'a DesignId,
    control: &'a DesignUnitId,
    treatment: &'a DesignUnitId,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

impl<'de> Deserialize<'de> for TwoUnits {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let design = DeserializedTwoUnits::deserialize(deserializer)?;
        Self::from_parts(
            design.id,
            design.control,
            design.treatment,
            design.meta,
            design.description,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedTwoUnits {
    id: DesignId,
    control: DesignUnitId,
    treatment: DesignUnitId,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::TwoUnits;
    use crate::design::{DesignId, DesignUnitId};

    #[test]
    fn requires_distinct_units() {
        let unit = DesignUnitId::new("UNIT1").unwrap();
        assert!(
            TwoUnits::new(
                DesignId::new("DES1").unwrap(),
                unit.clone(),
                unit,
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }
}
