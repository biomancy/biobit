use eyre::{Result, bail};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

/// An immutable ordered map whose inputs must not repeat a key.
///
/// Unlike [`BTreeMap::from_iter`] and `BTreeMap` deserialization, construction
/// rejects duplicate keys instead of silently retaining one of their values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct UniqueMap<K, V>(BTreeMap<K, V>);

impl<K, V> Default for UniqueMap<K, V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K, V> UniqueMap<K, V>
where
    K: Ord + Display,
{
    /// Collects entries while rejecting duplicate keys.
    pub(crate) fn try_from_iter(entries: impl IntoIterator<Item = (K, V)>) -> Result<Self> {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            match values.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(entry) => {
                    bail!("Map must not contain duplicate key '{}'", entry.key());
                }
            }
        }
        Ok(Self(values))
    }

    /// Consumes this map and returns its ordered entries.
    pub(crate) fn into_inner(self) -> BTreeMap<K, V> {
        self.0
    }
}

impl<K, V> From<BTreeMap<K, V>> for UniqueMap<K, V> {
    fn from(values: BTreeMap<K, V>) -> Self {
        Self(values)
    }
}

impl<K, V> AsRef<BTreeMap<K, V>> for UniqueMap<K, V> {
    fn as_ref(&self) -> &BTreeMap<K, V> {
        &self.0
    }
}

impl<K, V> Deref for UniqueMap<K, V> {
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// `BTreeMap` deserialization silently overwrites duplicate map keys. This
// visitor preserves the stricter ingestion contract of `UniqueMap`.
impl<'de, K, V> Deserialize<'de> for UniqueMap<K, V>
where
    K: Deserialize<'de> + Ord + Display,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UniqueMapVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K, V> Visitor<'de> for UniqueMapVisitor<K, V>
        where
            K: Deserialize<'de> + Ord + Display,
            V: Deserialize<'de>,
        {
            type Value = UniqueMap<K, V>;

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
                Ok(UniqueMap(values))
            }
        }

        deserializer.deserialize_map(UniqueMapVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::UniqueMap;

    #[test]
    fn rejects_duplicate_iterator_keys() {
        assert!(UniqueMap::try_from_iter([("key", 1), ("key", 2)]).is_err());
    }

    #[test]
    fn rejects_duplicate_serialized_keys() {
        assert!(
            serde_json::from_str::<UniqueMap<String, bool>>(r#"{"key":true,"key":false}"#).is_err()
        );
    }
}
