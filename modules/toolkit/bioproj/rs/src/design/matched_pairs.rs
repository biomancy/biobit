use super::{Designs, UnitId};
use crate::primitives::define_entity_id;
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text};
use std::collections::BTreeSet;

define_entity_id!(
    MatchedPairsId,
    "The identifier of a [`crate::design::MatchedPairs`] design."
);

/// An ordered correspondence between two units.
///
/// The order is preserved in equality: `(A, B)` differs from
/// `(B, A)`. It does not assign control or treatment labels to the members.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct MatchedPair {
    first: UnitId,
    second: UnitId,
}

impl MatchedPair {
    /// Creates an ordered pair of distinct units.
    pub fn new(first: UnitId, second: UnitId) -> Result<Self> {
        if first == second {
            return Err(Error::invalid(
                "MatchedPair",
                "first and second must be distinct",
            ));
        }
        Ok(Self { first, second })
    }

    /// Returns the first unit in this ordered pair.
    pub fn first(&self) -> &UnitId {
        &self.first
    }

    /// Returns the second unit in this ordered pair.
    pub fn second(&self) -> &UnitId {
        &self.second
    }
}

/// A collection of ordered, one-to-one matched unit pairs.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct MatchedPairs {
    id: MatchedPairsId,
    name: Text,
    pairs: NonEmpty<OrderedSet<MatchedPair>>,
    meta: Meta,
    description: Option<Text>,
}

impl MatchedPairs {
    /// Creates a non-empty collection of ordered, disjoint matched pairs.
    pub fn new(
        id: MatchedPairsId,
        name: Text,
        pairs: NonEmpty<OrderedSet<MatchedPair>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Result<Self> {
        let mut members = BTreeSet::new();
        for pair in &pairs {
            for unit_id in [pair.first(), pair.second()] {
                if !members.insert(unit_id) {
                    return Err(Error::invalid(
                        "MatchedPairs::pairs",
                        format!("must not use Unit '{unit_id}' in more than one pair"),
                    ));
                }
            }
        }
        Ok(Self {
            id,
            name,
            pairs,
            meta,
            description,
        })
    }

    /// Returns this design's identifier.
    pub fn id(&self) -> &MatchedPairsId {
        &self.id
    }

    /// Returns this design's non-blank printable ASCII name.
    pub fn name(&self) -> &Text {
        &self.name
    }

    /// Returns the ordered matched pairs.
    pub fn pairs(&self) -> &NonEmpty<OrderedSet<MatchedPair>> {
        &self.pairs
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
            self.pairs
                .iter()
                .flat_map(|pair| [("pairs", pair.first()), ("pairs", pair.second())]),
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{MatchedPair, MatchedPairs, MatchedPairsId};
    use crate::design::UnitId;
    use crate::{NonEmpty, Text};

    fn pair(first: &str, second: &str) -> MatchedPair {
        MatchedPair::new(UnitId::new(first).unwrap(), UnitId::new(second).unwrap()).unwrap()
    }

    #[test]
    fn treats_reversed_pairs_as_distinct() {
        let forward = MatchedPairs::new(
            MatchedPairsId::new("DES1").unwrap(),
            Text::new("Matched measurements").unwrap(),
            NonEmpty::try_from_iter([pair("UNIT_A", "UNIT_B")]).unwrap(),
            Default::default(),
            None,
        )
        .unwrap();
        let reverse = MatchedPairs::new(
            MatchedPairsId::new("DES1").unwrap(),
            Text::new("Matched measurements").unwrap(),
            NonEmpty::try_from_iter([pair("UNIT_B", "UNIT_A")]).unwrap(),
            Default::default(),
            None,
        )
        .unwrap();

        assert_ne!(forward, reverse);
    }

    #[test]
    fn rejects_a_unit_in_more_than_one_pair() {
        let error = MatchedPairs::new(
            MatchedPairsId::new("DES1").unwrap(),
            Text::new("Matched measurements").unwrap(),
            NonEmpty::try_from_iter([pair("A", "B"), pair("A", "C")]).unwrap(),
            Default::default(),
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("more than one pair"));
    }
}
