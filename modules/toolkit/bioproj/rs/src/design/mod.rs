//! Experimental-design topology over a provenance graph.

mod core;
pub mod matched_pairs;
pub mod two_groups;
pub mod two_units;
pub mod unit;
pub mod unit_set;

use crate::UntypedId;
use crate::primitives::define_entity_id;
use crate::provenance::Provenance;
use crate::validation;
use eyre::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

pub use matched_pairs::{MatchedPair, MatchedPairs};
pub use two_groups::TwoGroups;
pub use two_units::TwoUnits;
pub use unit::{DesignUnit, DesignUnitId};
pub use unit_set::UnitSet;

define_entity_id!(DesignId, "The identifier of a [`crate::design::Design`].");

/// A concrete experimental-design topology.
///
/// Each variant describes only the relationships among logical design units;
/// it does not prescribe a model, algorithm, or software parameters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, derive_more::From)]
#[serde(tag = "type")]
pub enum Design {
    /// A direct contrast between one control and one treatment unit.
    TwoUnits(TwoUnits),
    /// A two-arm contrast without required pairwise correspondence.
    TwoGroups(TwoGroups),
    /// Explicit ordered correspondence between pairs of units.
    MatchedPairs(MatchedPairs),
    /// An unordered collection of units for a joint analysis.
    UnitSet(UnitSet),
}

impl Design {
    /// Returns this design's identifier.
    pub fn id(&self) -> &DesignId {
        match self {
            Self::TwoUnits(design) => design.id(),
            Self::TwoGroups(design) => design.id(),
            Self::MatchedPairs(design) => design.id(),
            Self::UnitSet(design) => design.id(),
        }
    }

    fn validate_references(&self, units: &BTreeMap<DesignUnitId, DesignUnit>) -> Result<()> {
        match self {
            Self::TwoUnits(design) => design.validate_references(units),
            Self::TwoGroups(design) => design.validate_references(units),
            Self::MatchedPairs(design) => design.validate_references(units),
            Self::UnitSet(design) => design.validate_references(units),
        }
    }
}

/// A resolved collection of design units and experimental designs.
///
/// Construction and deserialization require parent [`Provenance`] so every
/// acquisition reference in a [`DesignUnit`] can be checked against released
/// provenance graph. The collection remains independently usable, while
/// [`crate::Project`] owns it together with provenance in a complete release.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Designs {
    units: BTreeMap<DesignUnitId, DesignUnit>,
    designs: BTreeMap<DesignId, Design>,
}

impl Designs {
    /// Constructs and validates designs against their parent provenance graph.
    pub fn new(
        provenance: &Provenance,
        units: impl IntoIterator<Item = DesignUnit>,
        designs: impl IntoIterator<Item = Design>,
    ) -> Result<Self> {
        let units: Vec<_> = units.into_iter().collect();
        let designs: Vec<_> = designs.into_iter().collect();

        validation::unique_ids(
            "provenance and design",
            provenance
                .ids()
                .chain(units.iter().map(|unit| unit.id().as_untyped()))
                .chain(designs.iter().map(|design| design.id().as_untyped())),
        )?;

        let units = units
            .into_iter()
            .map(|unit| (unit.id().clone(), unit))
            .collect();
        let designs = designs
            .into_iter()
            .map(|design| (design.id().clone(), design))
            .collect();

        let result = Self { units, designs };
        result.validate_against(provenance)?;

        Ok(result)
    }

    /// Iterates over all design units.
    pub fn units(&self) -> impl ExactSizeIterator<Item = &DesignUnit> + '_ {
        self.units.values()
    }

    /// Iterates over all experimental designs.
    pub fn designs(&self) -> impl ExactSizeIterator<Item = &Design> + '_ {
        self.designs.values()
    }

    /// Finds a design unit by its typed ID.
    pub fn unit(&self, id: &DesignUnitId) -> Option<&DesignUnit> {
        self.units.get(id)
    }

    /// Finds an experimental design by its typed ID.
    pub fn design(&self, id: &DesignId) -> Option<&Design> {
        self.designs.get(id)
    }

    /// Validates this collection against its parent provenance graph.
    pub(crate) fn validate_against(&self, provenance: &Provenance) -> Result<()> {
        validation::unique_ids("provenance and design", provenance.ids().chain(self.ids()))?;
        validate_unit_references(&self.units, provenance)?;
        validate_design_references(&self.designs, &self.units)
    }

    /// Iterates over IDs occupied by design units and designs.
    pub(crate) fn ids(&self) -> impl Iterator<Item = &UntypedId> {
        self.units
            .values()
            .map(|unit| unit.id().as_untyped())
            .chain(self.designs.values().map(|design| design.id().as_untyped()))
    }

    /// Deserializes and validates a design payload using parent provenance.
    ///
    /// A standalone `Deserialize` implementation would lack the acquisitions needed
    /// to validate design-unit references. This explicit method provides that
    /// context when deserializing the domain on its own.
    pub fn deserialize_with_provenance<'de, D>(
        provenance: &Provenance,
        deserializer: D,
    ) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        UnresolvedDesigns::deserialize(deserializer)?
            .resolve(provenance)
            .map_err(serde::de::Error::custom)
    }
}

fn validate_unit_references(
    units: &BTreeMap<DesignUnitId, DesignUnit>,
    provenance: &Provenance,
) -> Result<()> {
    for unit in units.values() {
        for acquisition_id in unit.acquisitions() {
            match provenance.get(acquisition_id) {
                Some(Ok(_)) => {}
                Some(Err(_)) => bail!(
                    "DesignUnit '{}' Acquisition '{}' resolves to a different type",
                    unit.id(),
                    acquisition_id.as_untyped()
                ),
                None => bail!(
                    "DesignUnit '{}' references unknown Acquisition '{}'",
                    unit.id(),
                    acquisition_id.as_untyped()
                ),
            }
        }
    }
    Ok(())
}

fn validate_design_references(
    designs: &BTreeMap<DesignId, Design>,
    units: &BTreeMap<DesignUnitId, DesignUnit>,
) -> Result<()> {
    for design in designs.values() {
        design.validate_references(units)?;
    }
    Ok(())
}

#[derive(Serialize)]
struct SerializedDesigns<'a> {
    units: Vec<&'a DesignUnit>,
    designs: Vec<&'a Design>,
}

impl Serialize for Designs {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedDesigns {
            units: self.units.values().collect(),
            designs: self.designs.values().collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedDesigns {
    units: Vec<unit::UnresolvedDesignUnit>,
    designs: Vec<Design>,
}

impl UnresolvedDesigns {
    pub(crate) fn resolve(self, provenance: &Provenance) -> Result<Designs> {
        let units = self
            .units
            .into_iter()
            .map(|unit| unit.resolve(provenance))
            .collect::<Result<Vec<_>>>()?;
        Designs::new(provenance, units, self.designs)
    }
}

#[cfg(test)]
mod tests {
    use super::{Design, DesignId, DesignUnit, DesignUnitId, Designs, TwoGroups, UnitSet};
    use crate::design::matched_pairs::MatchedPairs;
    use crate::design::two_units::TwoUnits;
    use crate::provenance::acquisition::illumina::{SingleEndSequencing, SingleEndSequencingId};
    use crate::provenance::library::p5p7;
    use crate::provenance::{
        Acquisition, AcquisitionId, Library, Provenance, Sample, SampleId, Source, SourceId,
    };

    fn provenance() -> Provenance {
        let source_id = SourceId::new("SRC1").unwrap();
        let sample_id = SampleId::new("SMP1").unwrap();
        let library_id = p5p7::LibraryId::new("LIB1").unwrap();
        let acquisition_one_id = SingleEndSequencingId::new("ACQ1").unwrap();
        let acquisition_two_id = SingleEndSequencingId::new("ACQ2").unwrap();

        Provenance::new(
            [Source::new(
                source_id.clone(),
                "Homo sapiens",
                Default::default(),
                None::<String>,
            )
            .unwrap()],
            [Sample::new(
                sample_id.clone(),
                [source_id],
                Default::default(),
                None::<String>,
            )
            .unwrap()],
            [Library::P5P7(
                p5p7::Library::new(
                    library_id.clone(),
                    [sample_id],
                    p5p7::Input::FromDna,
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            )],
            [
                Acquisition::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        acquisition_one_id,
                        [library_id.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Acquisition::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        acquisition_two_id,
                        [library_id],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap()
    }

    fn unit(id: &str, acquisition: &str) -> DesignUnit {
        DesignUnit::new(
            DesignUnitId::new(id).unwrap(),
            [AcquisitionId::IlluminaSingleEndSequencing(
                SingleEndSequencingId::new(acquisition).unwrap(),
            )],
            Default::default(),
            None::<String>,
        )
        .unwrap()
    }

    #[test]
    fn validates_acquisition_and_design_unit_references() {
        let provenance = provenance();
        let designs = Designs::new(
            &provenance,
            [unit("UNIT_CTRL", "ACQ1"), unit("UNIT_TREAT", "ACQ2")],
            [Design::TwoUnits(
                TwoUnits::new(
                    DesignId::new("DES1").unwrap(),
                    DesignUnitId::new("UNIT_CTRL").unwrap(),
                    DesignUnitId::new("UNIT_TREAT").unwrap(),
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(designs.units().len(), 2);
        assert_eq!(designs.designs().len(), 1);

        assert!(
            Designs::new(
                &provenance,
                [unit("UNIT_BAD", "UNKNOWN")],
                Vec::<Design>::new(),
            )
            .is_err()
        );

        assert!(
            Designs::new(
                &provenance,
                [unit("UNIT_CTRL", "ACQ1")],
                [Design::TwoUnits(
                    TwoUnits::new(
                        DesignId::new("DES1").unwrap(),
                        DesignUnitId::new("UNIT_CTRL").unwrap(),
                        DesignUnitId::new("UNIT_MISSING").unwrap(),
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                )],
            )
            .is_err()
        );
    }

    #[test]
    fn deserializes_ordered_matched_pairs_with_parent_provenance() {
        let provenance = provenance();
        let mut deserializer = serde_json::Deserializer::from_str(
            r#"{
                "units": [
                    {"id": "UNIT_A", "acquisitions": ["ACQ1"]},
                    {"id": "UNIT_B", "acquisitions": ["ACQ2"]}
                ],
                "designs": [
                    {
                        "type": "MatchedPairs",
                        "id": "DES1",
                        "pairs": [["UNIT_B", "UNIT_A"]]
                    }
                ]
            }"#,
        );
        let designs = Designs::deserialize_with_provenance(&provenance, &mut deserializer).unwrap();

        let Some(Design::MatchedPairs(design)) = designs.design(&DesignId::new("DES1").unwrap())
        else {
            panic!("serialized matched pairs resolved to a different design type");
        };
        let pair = design.pairs().as_ref().first().unwrap();
        assert_eq!(pair.first().as_untyped().as_str(), "UNIT_B");
        assert_eq!(pair.second().as_untyped().as_str(), "UNIT_A");
    }

    #[test]
    fn rejects_design_ids_already_used_by_provenance() {
        let provenance = provenance();
        assert!(
            Designs::new(
                &provenance,
                [unit("UNIT_CTRL", "ACQ1"), unit("UNIT_TREAT", "ACQ2")],
                [Design::TwoUnits(
                    TwoUnits::new(
                        DesignId::new("ACQ1").unwrap(),
                        DesignUnitId::new("UNIT_CTRL").unwrap(),
                        DesignUnitId::new("UNIT_TREAT").unwrap(),
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                )],
            )
            .is_err()
        );
    }

    #[test]
    fn serializes_deterministically() {
        let provenance = provenance();
        let designs = Designs::new(
            &provenance,
            [unit("UNIT_B", "ACQ2"), unit("UNIT_A", "ACQ1")],
            [Design::MatchedPairs(
                MatchedPairs::new(
                    DesignId::new("DES1").unwrap(),
                    [(
                        DesignUnitId::new("UNIT_B").unwrap(),
                        DesignUnitId::new("UNIT_A").unwrap(),
                    )],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&designs).unwrap(),
            r#"{"units":[{"id":"UNIT_A","acquisitions":["ACQ1"]},{"id":"UNIT_B","acquisitions":["ACQ2"]}],"designs":[{"type":"MatchedPairs","id":"DES1","pairs":[["UNIT_B","UNIT_A"]]}]}"#
        );
    }

    #[test]
    fn uses_internally_tagged_design_dispatch() {
        let Design::TwoUnits(design) = serde_json::from_str::<Design>(
            r#"{
                "type": "TwoUnits",
                "id": "DES1",
                "control": "UNIT_CTRL",
                "treatment": "UNIT_TREAT"
            }"#,
        )
        .unwrap() else {
            panic!("serialized design resolved to a different design type");
        };
        assert_eq!(design.id().as_untyped().as_str(), "DES1");

        assert!(serde_json::from_str::<Design>(r#"{"type":"Unknown","id":"DES1"}"#).is_err());
        assert!(
            serde_json::from_str::<Design>(
                r#"{"type":"TwoUnits","id":"DES1","control":"UNIT_CTRL","treatment":"UNIT_TREAT","extra":true}"#
            )
            .is_err()
        );
    }

    #[test]
    fn round_trips_each_flat_tagged_variant() {
        let unit_a = DesignUnitId::new("UNIT_A").unwrap();
        let unit_b = DesignUnitId::new("UNIT_B").unwrap();
        let designs = [
            (
                "TwoUnits",
                Design::TwoUnits(
                    TwoUnits::new(
                        DesignId::new("DES_TWO_UNITS").unwrap(),
                        unit_a.clone(),
                        unit_b.clone(),
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            (
                "TwoGroups",
                Design::TwoGroups(
                    TwoGroups::new(
                        DesignId::new("DES_TWO_GROUPS").unwrap(),
                        [unit_a.clone()],
                        [unit_b.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            (
                "MatchedPairs",
                Design::MatchedPairs(
                    MatchedPairs::new(
                        DesignId::new("DES_PAIRS").unwrap(),
                        [(unit_a.clone(), unit_b.clone())],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            (
                "UnitSet",
                Design::UnitSet(
                    UnitSet::new(
                        DesignId::new("DES_SET").unwrap(),
                        [unit_a, unit_b],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
        ];

        for (kind, design) in designs {
            let serialized = serde_json::to_string(&design).unwrap();
            let value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
            assert_eq!(value["type"], kind);
            assert_eq!(serde_json::from_str::<Design>(&serialized).unwrap(), design);
        }
    }
}
