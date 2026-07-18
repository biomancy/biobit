use super::DesignUnit;
use super::core::DesignCore;
use super::{DesignId, DesignUnitId};
use crate::{Id, Meta, MetaVal};
use eyre::{Result, bail, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

/// An ordered correspondence between two design units.
///
/// The order is preserved on the wire and in equality: `(A, B)` differs from
/// `(B, A)`. It does not assign control or treatment labels to the members.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MatchedPair {
    first: DesignUnitId,
    second: DesignUnitId,
}

impl MatchedPair {
    /// Creates an ordered pair of distinct design units.
    pub fn new(first: DesignUnitId, second: DesignUnitId) -> Result<Self> {
        ensure!(
            first != second,
            "MatchedPair::first and MatchedPair::second must be distinct"
        );
        Ok(Self { first, second })
    }

    /// Returns the first design unit in this ordered pair.
    pub fn first(&self) -> &DesignUnitId {
        &self.first
    }

    /// Returns the second design unit in this ordered pair.
    pub fn second(&self) -> &DesignUnitId {
        &self.second
    }
}

impl Serialize for MatchedPair {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&self.first, &self.second).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MatchedPair {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [first, second] = <[DesignUnitId; 2]>::deserialize(deserializer)?;
        Self::new(first, second).map_err(serde::de::Error::custom)
    }
}

/// A collection of ordered, one-to-one matched design-unit pairs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchedPairs {
    core: DesignCore,
    pairs: BTreeSet<MatchedPair>,
}

impl MatchedPairs {
    /// Creates a non-empty collection of ordered, disjoint matched pairs.
    pub fn new(
        id: DesignId,
        pairs: impl IntoIterator<Item = (DesignUnitId, DesignUnitId)>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let pairs = pairs
            .into_iter()
            .map(|(first, second)| MatchedPair::new(first, second))
            .collect::<Result<Vec<_>>>()?;
        Self::from_parts(id, pairs, Meta::new(meta)?, description.map(Into::into))
    }

    fn from_parts(
        id: DesignId,
        pairs: impl IntoIterator<Item = MatchedPair>,
        meta: Meta,
        description: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            core: DesignCore {
                id,
                meta,
                description,
            },
            pairs: collect_pairs(pairs)?,
        })
    }

    /// Returns this design's identifier.
    pub fn id(&self) -> &DesignId {
        &self.core.id
    }

    /// Returns the ordered matched pairs.
    pub fn pairs(&self) -> &BTreeSet<MatchedPair> {
        &self.pairs
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
        for pair in self.pairs() {
            for unit_id in [pair.first(), pair.second()] {
                if !units.contains_key(unit_id) {
                    bail!(
                        "MatchedPairs Design '{}' references unknown DesignUnit '{unit_id}'",
                        self.id()
                    );
                }
            }
        }
        Ok(())
    }
}

impl AsRef<Id> for MatchedPairs {
    fn as_ref(&self) -> &Id {
        self.id().as_id()
    }
}

fn collect_pairs(pairs: impl IntoIterator<Item = MatchedPair>) -> Result<BTreeSet<MatchedPair>> {
    let mut result = BTreeSet::new();
    let mut members = BTreeSet::new();

    for pair in pairs {
        if !result.insert(pair.clone()) {
            bail!(
                "MatchedPairs::pairs must not contain duplicate pair ('{}', '{}')",
                pair.first(),
                pair.second()
            );
        }

        for unit_id in [pair.first(), pair.second()] {
            if !members.insert(unit_id.clone()) {
                bail!(
                    "MatchedPairs::pairs must not use DesignUnit '{unit_id}' in more than one pair"
                );
            }
        }
    }

    ensure!(!result.is_empty(), "MatchedPairs::pairs must not be empty");
    Ok(result)
}

impl Serialize for MatchedPairs {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedMatchedPairs {
            id: self.id(),
            pairs: self.pairs(),
            meta: (!self.core.meta.is_empty()).then_some(&self.core.meta),
            description: self.core.description.as_deref(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedMatchedPairs<'a> {
    id: &'a DesignId,
    pairs: &'a BTreeSet<MatchedPair>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<&'a Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

impl<'de> Deserialize<'de> for MatchedPairs {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let design = DeserializedMatchedPairs::deserialize(deserializer)?;
        Self::from_parts(design.id, design.pairs, design.meta, design.description)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedMatchedPairs {
    id: DesignId,
    pairs: Vec<MatchedPair>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::MatchedPairs;
    use crate::design::{DesignId, DesignUnitId};

    #[test]
    fn preserves_pair_order_in_json() {
        let design = MatchedPairs::new(
            DesignId::new("DES1").unwrap(),
            [(
                DesignUnitId::new("UNIT_B").unwrap(),
                DesignUnitId::new("UNIT_A").unwrap(),
            )],
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&super::super::Design::MatchedPairs(design)).unwrap(),
            r#"{"type":"MatchedPairs","id":"DES1","pairs":[["UNIT_B","UNIT_A"]]}"#
        );
    }

    #[test]
    fn treats_reversed_pairs_as_distinct() {
        let forward = MatchedPairs::new(
            DesignId::new("DES1").unwrap(),
            [(
                DesignUnitId::new("UNIT_A").unwrap(),
                DesignUnitId::new("UNIT_B").unwrap(),
            )],
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();
        let reverse = MatchedPairs::new(
            DesignId::new("DES1").unwrap(),
            [(
                DesignUnitId::new("UNIT_B").unwrap(),
                DesignUnitId::new("UNIT_A").unwrap(),
            )],
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        assert_ne!(forward, reverse);
    }

    #[test]
    fn requires_each_unit_to_be_in_at_most_one_pair() {
        let unit_a = DesignUnitId::new("UNIT_A").unwrap();
        assert!(
            MatchedPairs::new(
                DesignId::new("DES1").unwrap(),
                [
                    (unit_a.clone(), DesignUnitId::new("UNIT_B").unwrap()),
                    (unit_a, DesignUnitId::new("UNIT_C").unwrap()),
                ],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }
}
