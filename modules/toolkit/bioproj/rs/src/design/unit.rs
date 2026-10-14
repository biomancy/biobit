use super::Designs;
use crate::primitives::define_entity_id;
use crate::provenance::{AcquisitionId, Provenance};
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text};

define_entity_id!(UnitId, "The identifier of a [`crate::design::Unit`].");

/// A named, non-empty collection of acquisitions pooled for computation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Unit {
    id: UnitId,
    acquisitions: NonEmpty<OrderedSet<AcquisitionId>>,
    meta: Meta,
    description: Option<Text>,
}

impl Unit {
    /// Creates a unit from one or more acquisitions.
    pub fn new(
        id: UnitId,
        acquisitions: NonEmpty<OrderedSet<AcquisitionId>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            acquisitions,
            meta,
            description,
        }
    }

    /// Returns this unit's identifier.
    pub fn id(&self) -> &UnitId {
        &self.id
    }

    /// Returns the acquisitions logically pooled in this unit.
    pub fn acquisitions(&self) -> &NonEmpty<OrderedSet<AcquisitionId>> {
        &self.acquisitions
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }

    /// Validates every Unit-to-Acquisition edge in a design domain.
    pub(crate) fn validate_against(provenance: &Provenance, designs: &Designs) -> Result<()> {
        for unit in designs.units() {
            for acquisition_id in unit.acquisitions() {
                match provenance.get(&acquisition_id.as_ref()) {
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error),
                    None => {
                        return Err(Error::missing(
                            "Unit",
                            unit.id().as_untyped(),
                            "acquisitions",
                            "Acquisition",
                            acquisition_id.as_untyped(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
