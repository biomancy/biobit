use super::NonEmpty;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::ops::Deref;

/// A scalar metadata value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaVal {
    /// A free-form string value.
    String(String),
    /// A boolean value.
    Bool(bool),
}

impl From<String> for MetaVal {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MetaVal {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<bool> for MetaVal {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

/// Auxiliary, non-structural annotations associated with a `bioproj` entity.
///
/// Metadata values are limited to strings and booleans. `Meta` must not define
/// an entity's identity, graph edges, compatibility, or other validated model
/// properties; those belong to explicit fields and tagged variants. Keys must
/// be non-empty and unique.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Meta(BTreeMap<NonEmpty<String>, MetaVal>);

impl Meta {
    /// Returns whether this metadata map has no entries.
    /// Used for skipping the serialization of empty metadata.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes this metadata map and returns its ordered entries.
    pub fn into_inner(self) -> BTreeMap<NonEmpty<String>, MetaVal> {
        self.0
    }
}

impl From<BTreeMap<NonEmpty<String>, MetaVal>> for Meta {
    fn from(values: BTreeMap<NonEmpty<String>, MetaVal>) -> Self {
        Self(values)
    }
}

impl AsRef<BTreeMap<NonEmpty<String>, MetaVal>> for Meta {
    fn as_ref(&self) -> &BTreeMap<NonEmpty<String>, MetaVal> {
        &self.0
    }
}

impl Deref for Meta {
    type Target = BTreeMap<NonEmpty<String>, MetaVal>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a Meta {
    type Item = (&'a NonEmpty<String>, &'a MetaVal);
    type IntoIter = std::collections::btree_map::Iter<'a, NonEmpty<String>, MetaVal>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// This cannot use the derived `BTreeMap` deserializer: duplicate keys in a
// serialized map overwrite their earlier values there. Released manifests
// reject that ambiguous input rather than applying last-write-wins semantics.
impl<'de> Deserialize<'de> for Meta {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetaVisitor;

        impl<'de> Visitor<'de> for MetaVisitor {
            type Value = Meta;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a metadata map with unique, non-empty keys")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Meta, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<NonEmpty<String>, MetaVal>()? {
                    if values.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!(
                            "metadata key '{key}' is not unique"
                        )));
                    }
                }
                Ok(Meta(values))
            }
        }

        deserializer.deserialize_map(MetaVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Meta, MetaVal, NonEmpty};
    use std::collections::BTreeMap;

    #[test]
    fn supports_string_and_boolean_values() {
        let meta = Meta::from(BTreeMap::from([
            (
                NonEmpty::new("organism".to_owned()).unwrap(),
                MetaVal::from("Homo sapiens"),
            ),
            (NonEmpty::new("paired".to_owned()).unwrap(), MetaVal::from(true)),
        ]));

        assert_eq!(
            meta.get("organism"),
            Some(&MetaVal::String("Homo sapiens".into()))
        );
        assert_eq!(meta.get("paired"), Some(&MetaVal::Bool(true)));
    }

    #[test]
    fn rejects_empty_keys() {
        assert!(serde_json::from_str::<Meta>(r#"{"":"value"}"#).is_err());
    }

    #[test]
    fn rejects_duplicate_keys_during_deserialization() {
        assert!(serde_json::from_str::<Meta>(r#"{"key":"first","key":"second"}"#).is_err());
    }

    #[test]
    fn serializes_as_a_string_keyed_map() {
        let meta = Meta::from(BTreeMap::from([(
            NonEmpty::new("key".to_owned()).unwrap(),
            MetaVal::from(true),
        )]));

        assert_eq!(serde_json::to_string(&meta).unwrap(), r#"{"key":true}"#);
    }
}
