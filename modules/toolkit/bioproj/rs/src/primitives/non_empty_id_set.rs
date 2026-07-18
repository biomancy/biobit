use crate::Id;
use eyre::{Result, bail, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;

/// A non-empty, duplicate-free set of IDs of one entity type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NonEmptyIdSet<T>(BTreeSet<T>);

impl<T> NonEmptyIdSet<T>
where
    T: Ord + AsRef<Id>,
{
    pub(crate) fn new(field: &str, values: impl IntoIterator<Item = T>) -> Result<Self> {
        let mut values_by_type = BTreeSet::new();
        let mut raw_ids = BTreeSet::new();
        for value in values {
            let id = value.as_ref();
            if !raw_ids.insert(id.clone()) {
                bail!("{field} must not contain duplicate ID '{id}'");
            }
            values_by_type.insert(value);
        }
        ensure!(!values_by_type.is_empty(), "{field} must not be empty");
        Ok(Self(values_by_type))
    }

    pub(crate) fn as_set(&self) -> &BTreeSet<T> {
        &self.0
    }
}

impl<T> Serialize for NonEmptyIdSet<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for NonEmptyIdSet<T>
where
    T: Deserialize<'de> + Ord + AsRef<Id>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Vec::<T>::deserialize(deserializer)?;
        Self::new("references", values).map_err(serde::de::Error::custom)
    }
}
