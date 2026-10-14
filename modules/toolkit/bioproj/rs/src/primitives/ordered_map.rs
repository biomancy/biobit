use crate::{Error, Result};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::collections::btree_map::{self, Entry};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

/// An ordered map that rejects duplicate input keys.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[serde(transparent)]
pub struct OrderedMap<K, V>(
    #[cfg_attr(feature = "bitcode", bitcode(bound_type = "BTreeMap<K, V>"))] BTreeMap<K, V>,
);

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Ord + Display,
{
    /// Builds a map and returns an error if a key repeats.
    pub fn try_from_iter(
        collection: &'static str,
        entries: impl IntoIterator<Item = (K, V)>,
    ) -> Result<Self> {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            match values.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(entry) => {
                    return Err(Error::DuplicateKey {
                        collection,
                        key: entry.key().to_string(),
                    });
                }
            }
        }
        Ok(Self(values))
    }
}

impl<K, V> OrderedMap<K, V> {
    /// Returns the entries in key order.
    pub fn into_inner(self) -> BTreeMap<K, V> {
        self.0
    }
}

impl<K, V> From<BTreeMap<K, V>> for OrderedMap<K, V> {
    fn from(values: BTreeMap<K, V>) -> Self {
        Self(values)
    }
}

impl<K, V> Borrow<BTreeMap<K, V>> for OrderedMap<K, V> {
    fn borrow(&self) -> &BTreeMap<K, V> {
        &self.0
    }
}

impl<K, V> Deref for OrderedMap<K, V> {
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = btree_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// `BTreeMap` deserialization silently overwrites duplicate map keys. This
// visitor preserves the stricter ingestion contract of `OrderedMap`.
impl<'de, K, V> Deserialize<'de> for OrderedMap<K, V>
where
    K: Deserialize<'de> + Ord + Display,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderedMapVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K, V> Visitor<'de> for OrderedMapVisitor<K, V>
        where
            K: Deserialize<'de> + Ord + Display,
            V: Deserialize<'de>,
        {
            type Value = OrderedMap<K, V>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map without duplicate keys")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    match values.entry(key) {
                        Entry::Vacant(entry) => {
                            entry.insert(value);
                        }
                        Entry::Occupied(entry) => {
                            return Err(serde::de::Error::custom(format!(
                                "Map must not contain duplicate key '{}'",
                                entry.key()
                            )));
                        }
                    }
                }
                Ok(OrderedMap(values))
            }
        }

        deserializer.deserialize_map(OrderedMapVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedMap;

    #[test]
    fn rejects_duplicate_iterator_keys() {
        assert!(OrderedMap::try_from_iter("map", [("key", 1), ("key", 2)]).is_err());
    }

    #[test]
    fn rejects_duplicate_serialized_keys() {
        assert!(
            serde_json::from_str::<OrderedMap<String, bool>>(r#"{"key":true,"key":false}"#)
                .is_err()
        );
    }
}
