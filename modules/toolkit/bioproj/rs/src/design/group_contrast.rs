use super::{Designs, UnitId};
use crate::primitives::define_entity_id;
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text};

define_entity_id!(
    GroupContrastId,
    "The identifier of a [`crate::design::GroupContrast`]."
);

/// A two-arm contrast without required pairwise correspondence between units.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct GroupContrast {
    id: GroupContrastId,
    name: Text,
    control: NonEmpty<OrderedSet<UnitId>>,
    treatment: NonEmpty<OrderedSet<UnitId>>,
    meta: Meta,
    description: Option<Text>,
}

impl GroupContrast {
    /// Creates a two-arm contrast from non-empty, disjoint groups of units.
    pub fn new(
        id: GroupContrastId,
        name: Text,
        control: NonEmpty<OrderedSet<UnitId>>,
        treatment: NonEmpty<OrderedSet<UnitId>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Result<Self> {
        if let Some(unit_id) = control.intersection(&treatment).next() {
            return Err(Error::invalid(
                "GroupContrast",
                format!("control and treatment must not share Unit '{unit_id}'"),
            ));
        }
        Ok(Self {
            id,
            name,
            control,
            treatment,
            meta,
            description,
        })
    }

    /// Returns this design's identifier.
    pub fn id(&self) -> &GroupContrastId {
        &self.id
    }

    /// Returns this design's non-blank printable ASCII name.
    pub fn name(&self) -> &Text {
        &self.name
    }

    /// Returns the control group.
    pub fn control(&self) -> &NonEmpty<OrderedSet<UnitId>> {
        &self.control
    }

    /// Returns the treatment group.
    pub fn treatment(&self) -> &NonEmpty<OrderedSet<UnitId>> {
        &self.treatment
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }

    pub(crate) fn validate_against(&self, designs: &Designs) -> Result<()> {
        designs.validate_units(
            self.id.as_untyped(),
            self.control
                .iter()
                .map(|id| ("control", id))
                .chain(self.treatment.iter().map(|id| ("treatment", id))),
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{GroupContrast, GroupContrastId};
    use crate::design::UnitId;
    use crate::{NonEmpty, Text};

    #[test]
    fn requires_disjoint_groups() {
        let unit = UnitId::new("UNIT1").unwrap();

        assert!(
            GroupContrast::new(
                GroupContrastId::new("DES1").unwrap(),
                Text::new("Control vs treatment groups").unwrap(),
                NonEmpty::try_from_iter([unit.clone()]).unwrap(),
                NonEmpty::try_from_iter([unit]).unwrap(),
                Default::default(),
                None,
            )
            .is_err()
        );
    }
}
