//! JSON 0.0.1 wire representations for the design domain.
//!
//! Loading resolves untyped Acquisition IDs and passes parsed fields through
//! domain constructors. Saving borrows the validated domain values.

use crate::design::{
    Design, Designs, GroupContrast, GroupContrastId, MatchedPair, MatchedPairs, MatchedPairsId,
    Unit, UnitContrast, UnitContrastId, UnitId, UnitSet, UnitSetId,
};
use crate::provenance::{AcquisitionId, Provenance};
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text, UntypedId};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitFromWire {
    id: UnitId,
    acquisitions: NonEmpty<OrderedSet<UntypedId>>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<Text>,
}

impl UnitFromWire {
    fn resolve(self, provenance: &Provenance) -> Result<Unit> {
        let Self {
            id,
            acquisitions,
            meta,
            description,
        } = self;
        let acquisitions = acquisitions
            .into_iter()
            .map(|acquisition_id| {
                provenance
                    .acquisition(&acquisition_id)
                    .map(|acquisition| acquisition.id().to_owned())
                    .ok_or_else(|| {
                        Error::missing(
                            "Unit",
                            id.as_untyped(),
                            "acquisitions",
                            "Acquisition",
                            &acquisition_id,
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Unit::new(
            id,
            NonEmpty::try_from_iter(acquisitions)?,
            meta,
            description,
        ))
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum DesignFromWire {
    UnitContrast {
        id: UnitContrastId,
        name: Text,
        control: UnitId,
        treatment: UnitId,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
    GroupContrast {
        id: GroupContrastId,
        name: Text,
        control: NonEmpty<OrderedSet<UnitId>>,
        treatment: NonEmpty<OrderedSet<UnitId>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
    MatchedPairs {
        id: MatchedPairsId,
        name: Text,
        pairs: NonEmpty<OrderedSet<[UnitId; 2]>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
    UnitSet {
        id: UnitSetId,
        name: Text,
        units: NonEmpty<OrderedSet<UnitId>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
}

impl DesignFromWire {
    fn resolve(self) -> Result<Design> {
        Ok(match self {
            Self::UnitContrast {
                id,
                name,
                control,
                treatment,
                meta,
                description,
            } => UnitContrast::new(id, name, control, treatment, meta, description)?.into(),
            Self::GroupContrast {
                id,
                name,
                control,
                treatment,
                meta,
                description,
            } => GroupContrast::new(id, name, control, treatment, meta, description)?.into(),
            Self::MatchedPairs {
                id,
                name,
                pairs,
                meta,
                description,
            } => {
                let pairs = pairs
                    .into_iter()
                    .map(|[first, second]| MatchedPair::new(first, second))
                    .collect::<Result<Vec<_>>>()?;
                MatchedPairs::new(id, name, NonEmpty::try_from_iter(pairs)?, meta, description)?
                    .into()
            }
            Self::UnitSet {
                id,
                name,
                units,
                meta,
                description,
            } => UnitSet::new(id, name, units, meta, description).into(),
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DesignsFromWire {
    units: Vec<UnitFromWire>,
    designs: Vec<DesignFromWire>,
}

impl DesignsFromWire {
    pub(super) fn resolve(self, provenance: &Provenance) -> Result<Designs> {
        let units = self
            .units
            .into_iter()
            .map(|unit| unit.resolve(provenance))
            .collect::<Result<Vec<_>>>()?;
        let designs = self
            .designs
            .into_iter()
            .map(DesignFromWire::resolve)
            .collect::<Result<Vec<_>>>()?;
        Designs::new(units, designs)
    }
}

#[derive(Serialize)]
struct UnitToWire<'a> {
    id: &'a UnitId,
    acquisitions: &'a NonEmpty<OrderedSet<AcquisitionId>>,
    #[serde(skip_serializing_if = "Meta::is_empty")]
    meta: &'a Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a Text>,
}

impl<'a> UnitToWire<'a> {
    fn new(unit: &'a Unit) -> Self {
        Self {
            id: unit.id(),
            acquisitions: unit.acquisitions(),
            meta: unit.meta(),
            description: unit.description(),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum DesignToWire<'a> {
    UnitContrast {
        id: &'a UnitContrastId,
        name: &'a Text,
        control: &'a UnitId,
        treatment: &'a UnitId,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
    GroupContrast {
        id: &'a GroupContrastId,
        name: &'a Text,
        control: &'a NonEmpty<OrderedSet<UnitId>>,
        treatment: &'a NonEmpty<OrderedSet<UnitId>>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
    MatchedPairs {
        id: &'a MatchedPairsId,
        name: &'a Text,
        pairs: PairsToWire<'a>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
    UnitSet {
        id: &'a UnitSetId,
        name: &'a Text,
        units: &'a NonEmpty<OrderedSet<UnitId>>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
}

impl<'a> DesignToWire<'a> {
    fn new(design: &'a Design) -> Self {
        match design {
            Design::UnitContrast(design) => Self::UnitContrast {
                id: design.id(),
                name: design.name(),
                control: design.control(),
                treatment: design.treatment(),
                meta: design.meta(),
                description: design.description(),
            },
            Design::GroupContrast(design) => Self::GroupContrast {
                id: design.id(),
                name: design.name(),
                control: design.control(),
                treatment: design.treatment(),
                meta: design.meta(),
                description: design.description(),
            },
            Design::MatchedPairs(design) => Self::MatchedPairs {
                id: design.id(),
                name: design.name(),
                pairs: PairsToWire(design.pairs()),
                meta: design.meta(),
                description: design.description(),
            },
            Design::UnitSet(design) => Self::UnitSet {
                id: design.id(),
                name: design.name(),
                units: design.units(),
                meta: design.meta(),
                description: design.description(),
            },
        }
    }
}

struct PairsToWire<'a>(&'a NonEmpty<OrderedSet<MatchedPair>>);

impl Serialize for PairsToWire<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter().map(|pair| (pair.first(), pair.second())))
    }
}

#[derive(Serialize)]
pub(super) struct DesignsToWire<'a> {
    units: Vec<UnitToWire<'a>>,
    designs: Vec<DesignToWire<'a>>,
}

impl<'a> DesignsToWire<'a> {
    pub(super) fn new(designs: &'a Designs) -> Self {
        Self {
            units: designs.units().map(UnitToWire::new).collect(),
            designs: designs.designs().map(DesignToWire::new).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DesignFromWire, DesignToWire};
    use crate::design::{Design, MatchedPair, MatchedPairs, MatchedPairsId, UnitId};
    use crate::{Error, NonEmpty, Text};

    #[test]
    fn preserves_ordered_pairs_in_json() {
        let pair = MatchedPair::new(
            UnitId::new("UNIT_B").unwrap(),
            UnitId::new("UNIT_A").unwrap(),
        )
        .unwrap();
        let design = Design::MatchedPairs(
            MatchedPairs::new(
                MatchedPairsId::new("DES1").unwrap(),
                Text::new("Matched measurements").unwrap(),
                NonEmpty::try_from_iter([pair]).unwrap(),
                Default::default(),
                None,
            )
            .unwrap(),
        );
        let json = serde_json::to_string(&DesignToWire::new(&design)).unwrap();
        assert_eq!(
            json,
            r#"{"type":"MatchedPairs","id":"DES1","name":"Matched measurements","pairs":[["UNIT_B","UNIT_A"]]}"#
        );
        let from_wire: DesignFromWire = serde_json::from_str(&json).unwrap();
        assert_eq!(from_wire.resolve().unwrap(), design);
    }

    #[test]
    fn loading_uses_domain_validation() {
        let from_wire: DesignFromWire = serde_json::from_str(
            r#"{"type":"UnitContrast","id":"DES1","name":"Control vs treatment","control":"UNIT1","treatment":"UNIT1"}"#,
        )
        .unwrap();
        assert!(matches!(
            from_wire.resolve(),
            Err(Error::InvalidValue { .. })
        ));
    }
}
