use super::{Designs, UnitId};
use crate::primitives::define_entity_id;
use crate::{Error, Meta, Result, Text};

define_entity_id!(
    UnitContrastId,
    "The identifier of a [`crate::design::UnitContrast`]."
);

/// A direct contrast between one control and one treatment unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitContrast {
    id: UnitContrastId,
    name: Text,
    control: UnitId,
    treatment: UnitId,
    meta: Meta,
    description: Option<Text>,
}

impl UnitContrast {
    /// Creates a direct contrast between two distinct units.
    pub fn new(
        id: UnitContrastId,
        name: Text,
        control: UnitId,
        treatment: UnitId,
        meta: Meta,
        description: Option<Text>,
    ) -> Result<Self> {
        if control == treatment {
            return Err(Error::invalid(
                "UnitContrast",
                "control and treatment must be distinct",
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
    pub fn id(&self) -> &UnitContrastId {
        &self.id
    }

    /// Returns this design's non-blank printable ASCII name.
    pub fn name(&self) -> &Text {
        &self.name
    }

    /// Returns the control unit.
    pub fn control(&self) -> &UnitId {
        &self.control
    }

    /// Returns the treatment unit.
    pub fn treatment(&self) -> &UnitId {
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
            [("control", self.control()), ("treatment", self.treatment())],
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{UnitContrast, UnitContrastId};
    use crate::Text;
    use crate::design::UnitId;

    #[test]
    fn requires_distinct_units() {
        let unit = UnitId::new("UNIT1").unwrap();
        assert!(
            UnitContrast::new(
                UnitContrastId::new("DES1").unwrap(),
                Text::new("Control vs treatment").unwrap(),
                unit.clone(),
                unit,
                Default::default(),
                None,
            )
            .is_err()
        );
    }
}
