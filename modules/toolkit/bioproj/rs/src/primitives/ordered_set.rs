use super::{IsEmpty, NonEmpty};
use crate::{Error, Result};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeSet, btree_set};
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::Deref;

/// An ordered set that rejects duplicate input values.
///
/// Empty sets are allowed. Use [`NonEmpty<OrderedSet<T>>`] when at least one
/// value is required. Serialization produces a sorted sequence.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[serde(transparent)]
pub struct OrderedSet<T>(
    #[cfg_attr(feature = "bitcode", bitcode(bound_type = "BTreeSet<T>"))] BTreeSet<T>,
);

impl<T> Default for OrderedSet<T> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<T: Ord> OrderedSet<T> {
    /// Builds a set and returns an error if a value repeats.
    pub fn try_from_iter(values: impl IntoIterator<Item = T>) -> Result<Self> {
        let mut result = BTreeSet::new();
        for value in values {
            if !result.insert(value) {
                return Err(Error::DuplicateValue);
            }
        }
        Ok(Self(result))
    }
}

impl<T> OrderedSet<T> {
    /// Returns the values as an ordered set.
    pub fn into_inner(self) -> BTreeSet<T> {
        self.0
    }
}

impl<T> From<BTreeSet<T>> for OrderedSet<T> {
    fn from(values: BTreeSet<T>) -> Self {
        Self(values)
    }
}

impl<T> IsEmpty for OrderedSet<T> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Ord> NonEmpty<OrderedSet<T>> {
    /// Builds a non-empty set and returns an error if a value repeats.
    pub fn try_from_iter(values: impl IntoIterator<Item = T>) -> Result<Self> {
        Self::new(OrderedSet::try_from_iter(values)?)
    }
}

impl<T> Borrow<BTreeSet<T>> for OrderedSet<T> {
    fn borrow(&self) -> &BTreeSet<T> {
        &self.0
    }
}

impl<T> Deref for OrderedSet<T> {
    type Target = BTreeSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> IntoIterator for OrderedSet<T> {
    type Item = T;
    type IntoIter = btree_set::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a OrderedSet<T> {
    type Item = &'a T;
    type IntoIter = btree_set::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Inserting as the sequence is read preserves duplicate input for validation
// without allocating an intermediate Vec or letting BTreeSet discard it.
impl<'de, T> Deserialize<'de> for OrderedSet<T>
where
    T: Deserialize<'de> + Ord,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderedSetVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for OrderedSetVisitor<T>
        where
            T: Deserialize<'de> + Ord,
        {
            type Value = OrderedSet<T>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a sequence without duplicate elements")
            }

            fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = BTreeSet::new();
                while let Some(value) = sequence.next_element()? {
                    if !values.insert(value) {
                        return Err(serde::de::Error::custom(Error::DuplicateValue));
                    }
                }
                Ok(OrderedSet(values))
            }
        }

        deserializer.deserialize_seq(OrderedSetVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedSet;
    use crate::Error;

    #[cfg(feature = "bitcode")]
    #[test]
    fn bitcode_roundtrip() {
        for value in [
            OrderedSet::<u32>::default(),
            OrderedSet::try_from_iter([1, 2, 3]).unwrap(),
        ] {
            assert_eq!(
                bitcode::decode::<OrderedSet<u32>>(&bitcode::encode(&value)).unwrap(),
                value
            );
        }
    }

    #[test]
    fn rejects_duplicate_iterator_entries() {
        assert!(matches!(
            OrderedSet::try_from_iter([1, 2, 1]),
            Err(Error::DuplicateValue)
        ));
    }

    #[test]
    fn rejects_duplicate_serialized_entries() {
        let error = serde_json::from_str::<OrderedSet<u32>>("[1, 2, 1]").unwrap_err();
        assert!(error.to_string().contains("duplicate"));
    }

    #[test]
    fn allows_empty_sets_and_serializes_in_order() {
        let empty: OrderedSet<u32> = serde_json::from_str("[]").unwrap();
        assert!(empty.is_empty());
        assert!(OrderedSet::<u32>::try_from_iter([]).unwrap().is_empty());

        let values: OrderedSet<u32> = serde_json::from_str("[3, 1, 2]").unwrap();
        assert_eq!(serde_json::to_string(&values).unwrap(), "[1,2,3]");
        assert_eq!(values.into_iter().collect::<Vec<_>>(), [1, 2, 3]);
    }
}
