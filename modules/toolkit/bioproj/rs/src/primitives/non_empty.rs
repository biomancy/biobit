use crate::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

/// A type that can be empty.
pub trait IsEmpty {
    /// Returns whether the value is empty.
    fn is_empty(&self) -> bool;
}

impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}

impl<T> IsEmpty for BTreeSet<T> {
    fn is_empty(&self) -> bool {
        BTreeSet::is_empty(self)
    }
}

/// A value that is not empty.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct NonEmpty<T: IsEmpty>(T);

impl<T: IsEmpty> NonEmpty<T> {
    /// Creates a value, returning an error if it is empty.
    pub fn new(value: T) -> Result<Self> {
        if value.is_empty() {
            return Err(Error::EmptyValue);
        }
        Ok(Self(value))
    }

    /// Returns the wrapped value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Borrow<str> for NonEmpty<String> {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl<T: IsEmpty> Borrow<T> for NonEmpty<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: IsEmpty> Deref for NonEmpty<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: IsEmpty + IntoIterator> IntoIterator for NonEmpty<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a NonEmpty<T>
where
    T: IsEmpty,
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: IsEmpty + Display> Display for NonEmpty<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: IsEmpty + Serialize> Serialize for NonEmpty<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for NonEmpty<T>
where
    T: Deserialize<'de> + IsEmpty,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(T::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::NonEmpty;
    use crate::OrderedSet;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn rejects_empty_scalars_and_sets() {
        assert!(NonEmpty::new(String::new()).is_err());
        assert!(NonEmpty::new(BTreeSet::<String>::new()).is_err());
        assert!(NonEmpty::new(OrderedSet::<String>::default()).is_err());
    }

    #[test]
    fn deserialization_rejects_empty_sets() {
        assert!(serde_json::from_str::<NonEmpty<OrderedSet<String>>>(r#"[]"#).is_err());
    }

    #[test]
    fn serializes_transparently() {
        let value = NonEmpty::new("value".to_owned()).unwrap();
        assert_eq!(serde_json::to_string(&value).unwrap(), r#""value""#);
    }

    #[test]
    fn non_empty_strings_support_str_map_lookups() {
        let mut values = BTreeMap::new();
        values.insert(NonEmpty::new("value".to_owned()).unwrap(), true);

        assert_eq!(values.get("value"), Some(&true));
    }
}
