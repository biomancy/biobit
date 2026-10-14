use super::{Designs, UnitId};
use crate::primitives::define_entity_id;
use crate::{Meta, NonEmpty, OrderedSet, Result, Text};

define_entity_id!(
    UnitSetId,
    "The identifier of a [`crate::design::UnitSet`] design."
);

/// An unordered collection of units for a joint analysis.
///
/// This topology does not assert independence; Units may share Acquisitions.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct UnitSet {
    id: UnitSetId,
    name: Text,
    units: NonEmpty<OrderedSet<UnitId>>,
    meta: Meta,
    description: Option<Text>,
}

impl UnitSet {
    /// Creates a non-empty unordered collection of units.
    pub fn new(
        id: UnitSetId,
        name: Text,
        units: NonEmpty<OrderedSet<UnitId>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            name,
            units,
            meta,
            description,
        }
    }

    /// Returns this design's identifier.
    pub fn id(&self) -> &UnitSetId {
        &self.id
    }

    /// Returns this design's non-blank printable ASCII name.
    pub fn name(&self) -> &Text {
        &self.name
    }

    /// Returns the unordered units.
    pub fn units(&self) -> &NonEmpty<OrderedSet<UnitId>> {
        &self.units
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
            self.units.iter().map(|id| ("units", id)),
            false,
        )
    }
}
