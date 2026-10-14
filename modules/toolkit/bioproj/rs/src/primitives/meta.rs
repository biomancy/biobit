use super::{OrderedMap, Text};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeMap, btree_map};
use std::ops::Deref;

/// A metadata value: a string or a boolean.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, derive_more::From)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[serde(untagged)]
pub enum MetaVal {
    /// Non-empty printable ASCII text.
    String(Text),
    /// A boolean value.
    Bool(bool),
}

impl TryFrom<&str> for MetaVal {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Text::new(value).map(Self::String)
    }
}

impl TryFrom<String> for MetaVal {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Text::new(value).map(Self::String)
    }
}

/// Additional context for a project entity, such as treatments or preparation
/// kits.
///
/// Values are strings or booleans. Keys and string values are non-empty
/// printable ASCII text, and keys are unique. Identity, relationships, and
/// compatibility belong in the entity's explicit fields.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[serde(transparent)]
pub struct Meta(OrderedMap<Text, MetaVal>);

impl Meta {
    /// Builds metadata and returns an error if a key repeats.
    pub fn try_from_iter(entries: impl IntoIterator<Item = (Text, MetaVal)>) -> Result<Self> {
        OrderedMap::try_from_iter("meta", entries).map(Self)
    }

    /// Returns whether the metadata is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the entries in key order.
    pub fn into_inner(self) -> BTreeMap<Text, MetaVal> {
        self.0.into_inner()
    }
}

impl From<BTreeMap<Text, MetaVal>> for Meta {
    fn from(values: BTreeMap<Text, MetaVal>) -> Self {
        Self(values.into())
    }
}

impl Borrow<BTreeMap<Text, MetaVal>> for Meta {
    fn borrow(&self) -> &BTreeMap<Text, MetaVal> {
        self.0.borrow()
    }
}

impl Deref for Meta {
    type Target = BTreeMap<Text, MetaVal>;

    fn deref(&self) -> &Self::Target {
        self.0.borrow()
    }
}

impl IntoIterator for Meta {
    type Item = (Text, MetaVal);
    type IntoIter = btree_map::IntoIter<Text, MetaVal>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Meta {
    type Item = (&'a Text, &'a MetaVal);
    type IntoIter = btree_map::Iter<'a, Text, MetaVal>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{Meta, MetaVal, Text};
    use crate::Error;
    use std::collections::BTreeMap;

    #[cfg(feature = "bitcode")]
    #[test]
    fn bitcode_roundtrip() {
        for value in [
            MetaVal::try_from("Homo sapiens").unwrap(),
            MetaVal::Bool(false),
        ] {
            assert_eq!(
                bitcode::decode::<MetaVal>(&bitcode::encode(&value)).unwrap(),
                value
            );
        }

        let meta = Meta::try_from_iter([
            (
                Text::new("organism").unwrap(),
                MetaVal::try_from("Homo sapiens").unwrap(),
            ),
            (Text::new("control").unwrap(), MetaVal::Bool(true)),
        ])
        .unwrap();
        let values = vec![meta.clone(), Meta::default(), meta];
        assert_eq!(
            bitcode::decode::<Vec<Meta>>(&bitcode::encode(&values)).unwrap(),
            values
        );
    }

    #[test]
    fn supports_string_and_boolean_values() {
        let meta = Meta::try_from_iter([
            (
                Text::new("organism").unwrap(),
                MetaVal::try_from("Homo sapiens").unwrap(),
            ),
            (Text::new("paired").unwrap(), MetaVal::from(true)),
        ])
        .unwrap();

        assert_eq!(
            meta.get("organism"),
            Some(&MetaVal::String(Text::new("Homo sapiens").unwrap()))
        );
        assert_eq!(meta.get("paired"), Some(&MetaVal::Bool(true)));
    }

    #[test]
    fn rejects_duplicate_keys_during_deserialization() {
        assert!(serde_json::from_str::<Meta>(r#"{"key":"first","key":"second"}"#).is_err());
    }

    #[test]
    fn rejects_duplicate_iterator_keys() {
        let key = Text::new("condition").unwrap();
        let error = Meta::try_from_iter([
            (key.clone(), MetaVal::Bool(true)),
            (key, MetaVal::Bool(false)),
        ])
        .unwrap_err();
        assert!(
            matches!(error, Error::DuplicateKey { collection: "meta", key }
            if key == "condition")
        );
    }

    #[test]
    fn serializes_as_a_string_keyed_map() {
        let meta = Meta::from(BTreeMap::from([(
            Text::new("key").unwrap(),
            MetaVal::from(true),
        )]));

        assert_eq!(serde_json::to_string(&meta).unwrap(), r#"{"key":true}"#);
    }
}
