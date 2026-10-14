use super::{Unit, UnitId};
use super::{group_contrast, matched_pairs, unit_contrast, unit_set};
use crate::lookup;
use crate::primitives::{OrderedMap, define_entity_family};
use crate::primitives::{impl_checked_lookup, impl_direct_lookup, impl_variant_lookup};
use crate::provenance::Provenance;
use crate::{Error, Lookup, Result, Text, UntypedId};
use std::collections::BTreeMap;

define_entity_family! {
    /// A concrete experimental-design topology.
    ///
    /// Each variant describes only the relationships among logical units; it
    /// does not prescribe a model, algorithm, or software parameters.
    pub family Design {
        kind: DesignKind,
        id: DesignId,
        id_ref: DesignIdRef,
        kind_doc: "The concrete type of a design or design identifier.",
        variants: {
            /// A direct contrast between one control and one treatment unit.
            UnitContrast(unit_contrast::UnitContrast, unit_contrast::UnitContrastId),
            /// A two-arm contrast without required pairwise correspondence.
            GroupContrast(group_contrast::GroupContrast, group_contrast::GroupContrastId),
            /// Explicit ordered correspondence between pairs of units.
            MatchedPairs(matched_pairs::MatchedPairs, matched_pairs::MatchedPairsId),
            /// An unordered collection of units for a joint analysis.
            UnitSet(unit_set::UnitSet, unit_set::UnitSetId),
        }
    }
}

impl Design {
    /// Returns this design's required human-readable name.
    pub fn name(&self) -> &Text {
        match self {
            Self::UnitContrast(design) => design.name(),
            Self::GroupContrast(design) => design.name(),
            Self::MatchedPairs(design) => design.name(),
            Self::UnitSet(design) => design.name(),
        }
    }

    /// Validates every Design-to-Unit edge in a design domain.
    pub(crate) fn validate(designs: &Designs) -> Result<()> {
        for design in designs.designs() {
            match design {
                Self::UnitContrast(design) => design.validate_against(designs)?,
                Self::GroupContrast(design) => design.validate_against(designs)?,
                Self::MatchedPairs(design) => design.validate_against(designs)?,
                Self::UnitSet(design) => design.validate_against(designs)?,
            }
        }
        Ok(())
    }
}

/// Units and experimental designs.
///
/// Construction checks Design-to-Unit relationships. [`crate::Project`]
/// checks Unit-to-Acquisition relationships against [`crate::provenance::Provenance`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Designs {
    units: OrderedMap<UnitId, Unit>,
    designs: OrderedMap<UntypedId, Design>,
}

impl Designs {
    /// Constructs Designs and checks their Unit relationships.
    pub fn new(
        units: impl IntoIterator<Item = Unit>,
        designs: impl IntoIterator<Item = Design>,
    ) -> Result<Self> {
        let designs = Self {
            units: OrderedMap::try_from_iter(
                "design.units",
                units.into_iter().map(|unit| (unit.id().clone(), unit)),
            )?,
            designs: OrderedMap::try_from_iter(
                "design.designs",
                designs
                    .into_iter()
                    .map(|design| (design.id().as_untyped().clone(), design)),
            )?,
        };
        Design::validate(&designs)?;
        Ok(designs)
    }

    /// Iterates over all units.
    pub fn units(&self) -> impl ExactSizeIterator<Item = &Unit> + '_ {
        self.units.values()
    }

    /// Iterates over all experimental designs.
    pub fn designs(&self) -> impl ExactSizeIterator<Item = &Design> + '_ {
        self.designs.values()
    }

    /// Performs a lookup whose result is determined by its typed ID.
    pub fn get<'a, K: ?Sized>(
        &'a self,
        key: &K,
    ) -> lookup::OptResult<&'a <Self as Lookup<K>>::Target, <Self as Lookup<K>>::Err>
    where
        Self: Lookup<K>,
    {
        <Self as Lookup<K>>::lookup(self, key)
    }

    /// Finds a unit by its untyped ID.
    pub fn unit(&self, id: &UntypedId) -> Option<&Unit> {
        self.units.get(id)
    }

    /// Finds a design by its untyped ID.
    pub fn design(&self, id: &UntypedId) -> Option<&Design> {
        self.designs.get(id)
    }

    pub(super) fn validate_units<'a>(
        &self,
        design: &UntypedId,
        units: impl IntoIterator<Item = (&'static str, &'a UnitId)>,
        disjoint: bool,
    ) -> Result<()> {
        let mut acquisitions = BTreeMap::new();
        for (field, unit_id) in units {
            let unit = self.get(unit_id).transpose()?.ok_or_else(|| {
                Error::missing("Design", design, field, "Unit", unit_id.as_untyped())
            })?;
            if disjoint {
                for acquisition in unit.acquisitions() {
                    if let Some(previous) =
                        acquisitions.insert(acquisition.as_untyped().clone(), unit_id.clone())
                    {
                        return Err(Error::AcquisitionOverlap {
                            design: design.clone(),
                            acquisition: acquisition.as_untyped().clone(),
                            first_unit: previous.into_untyped(),
                            second_unit: unit_id.as_untyped().clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks Unit-to-Acquisition relationships against Provenance.
    pub(crate) fn validate_against(&self, provenance: &Provenance) -> Result<()> {
        Unit::validate_against(provenance, self)
    }
}

impl_direct_lookup!(Designs, UnitId, Unit, units);
impl_checked_lookup!(Designs, DesignId, Design, designs);
impl_checked_lookup!(Designs, DesignIdRef<'_>, Design, designs);

impl_variant_lookup!(
    Designs,
    unit_contrast::UnitContrastId,
    unit_contrast::UnitContrast,
    designs,
    Design::UnitContrast
);
impl_variant_lookup!(
    Designs,
    group_contrast::GroupContrastId,
    group_contrast::GroupContrast,
    designs,
    Design::GroupContrast
);
impl_variant_lookup!(
    Designs,
    matched_pairs::MatchedPairsId,
    matched_pairs::MatchedPairs,
    designs,
    Design::MatchedPairs
);
impl_variant_lookup!(
    Designs,
    unit_set::UnitSetId,
    unit_set::UnitSet,
    designs,
    Design::UnitSet
);
