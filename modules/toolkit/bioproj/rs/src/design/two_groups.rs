use super::DesignUnit;
use super::core::DesignCore;
use super::{DesignId, DesignUnitId};
use crate::{Meta, NonEmpty};
use eyre::{Result, bail};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

/// A two-arm contrast without required pairwise correspondence between units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwoGroups {
    core: DesignCore,
    control: NonEmpty<BTreeSet<DesignUnitId>>,
    treatment: NonEmpty<BTreeSet<DesignUnitId>>,
}

impl TwoGroups {
    /// Creates a two-arm contrast from non-empty, disjoint groups of units.
    pub fn new(
        id: DesignId,
        control: impl IntoIterator<Item = DesignUnitId>,
        treatment: impl IntoIterator<Item = DesignUnitId>,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let control = NonEmpty::try_from_iter(control)?;
        let treatment = NonEmpty::try_from_iter(treatment)?;
        validate_disjoint_groups(&control, &treatment)?;
        Ok(Self {
            core: DesignCore::new(id, meta, description)?,
            control,
            treatment,
        })
    }

    fn from_parts(
        id: DesignId,
        control: NonEmpty<BTreeSet<DesignUnitId>>,
        treatment: NonEmpty<BTreeSet<DesignUnitId>>,
        meta: Meta,
        description: Option<NonEmpty<String>>,
    ) -> Result<Self> {
        validate_disjoint_groups(&control, &treatment)?;
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

    /// Returns the control group.
    pub fn control(&self) -> &NonEmpty<BTreeSet<DesignUnitId>> {
        &self.control
    }

    /// Returns the treatment group.
    pub fn treatment(&self) -> &NonEmpty<BTreeSet<DesignUnitId>> {
        &self.treatment
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.core.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        self.core.description.as_ref()
    }

    pub(crate) fn validate_references(
        &self,
        units: &BTreeMap<DesignUnitId, DesignUnit>,
    ) -> Result<()> {
        for unit_id in self
            .control()
            .as_ref()
            .iter()
            .chain(self.treatment().as_ref())
        {
            if !units.contains_key(unit_id) {
                bail!(
                    "TwoGroups Design '{}' references unknown DesignUnit '{unit_id}'",
                    self.id()
                );
            }
        }
        Ok(())
    }
}

fn validate_disjoint_groups(
    control: &NonEmpty<BTreeSet<DesignUnitId>>,
    treatment: &NonEmpty<BTreeSet<DesignUnitId>>,
) -> Result<()> {
    if let Some(unit_id) = control.as_ref().intersection(treatment.as_ref()).next() {
        bail!("TwoGroups::control and TwoGroups::treatment must not share DesignUnit '{unit_id}'");
    }
    Ok(())
}

impl Serialize for TwoGroups {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedTwoGroups {
            id: self.id(),
            control: &self.control,
            treatment: &self.treatment,
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_ref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedTwoGroups<'a> {
    id: &'a DesignId,
    control: &'a NonEmpty<BTreeSet<DesignUnitId>>,
    treatment: &'a NonEmpty<BTreeSet<DesignUnitId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a NonEmpty<String>>,
}

impl<'de> Deserialize<'de> for TwoGroups {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let design = DeserializedTwoGroups::deserialize(deserializer)?;
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
struct DeserializedTwoGroups {
    id: DesignId,
    control: NonEmpty<BTreeSet<DesignUnitId>>,
    treatment: NonEmpty<BTreeSet<DesignUnitId>>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<NonEmpty<String>>,
}

#[cfg(test)]
mod tests {
    use super::TwoGroups;
    use crate::design::{DesignId, DesignUnitId};

    #[test]
    fn requires_non_empty_disjoint_groups() {
        let unit = DesignUnitId::new("UNIT1").unwrap();

        assert!(
            TwoGroups::new(
                DesignId::new("DES1").unwrap(),
                Vec::<DesignUnitId>::new(),
                [unit.clone()],
                Default::default(),
                None::<String>,
            )
            .is_err()
        );

        assert!(
            TwoGroups::new(
                DesignId::new("DES1").unwrap(),
                [unit.clone()],
                [unit],
                Default::default(),
                None::<String>,
            )
            .is_err()
        );
    }
}
