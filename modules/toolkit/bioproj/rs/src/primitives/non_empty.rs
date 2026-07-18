use eyre::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

/// A value that can report whether it is empty.
///
/// This deliberately small trait lets [`NonEmpty`] refine existing value and
/// collection types without teaching the domain model about each individual
/// empty-state representation.
pub trait IsEmpty {
    /// Returns whether this value has no contents.
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

/// A value whose type-specific empty state has been ruled out.
///
/// The wrapped value is exposed only immutably. This prevents callers from
/// invalidating the invariant after construction.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct NonEmpty<T: IsEmpty>(T);

impl<T: IsEmpty> NonEmpty<T> {
    /// Creates a non-empty value.
    pub fn new(value: T) -> Result<Self> {
        ensure!(!value.is_empty(), "Value must not be empty");
        Ok(Self(value))
    }

    /// Consumes this wrapper and returns the wrapped value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: IsEmpty> AsRef<T> for NonEmpty<T> {
    fn as_ref(&self) -> &T {
        &self.0
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

impl<'a, T> IntoIterator for &'a NonEmpty<T>
where
    T: IsEmpty,
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().into_iter()
    }
}

impl<T: IsEmpty + Display> Display for NonEmpty<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: IsEmpty + Serialize> Serialize for NonEmpty<T>
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + IsEmpty> Deserialize<'de> for NonEmpty<T>
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
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn rejects_empty_scalars_and_sets() {
        assert!(NonEmpty::new(String::new()).is_err());
        assert!(NonEmpty::new(BTreeSet::<String>::new()).is_err());
    }

    #[test]
    fn deserialization_preserves_non_empty_and_duplicate_invariants() {
        assert!(serde_json::from_str::<NonEmpty<String>>("\"\"").is_err());
        assert!(serde_json::from_str::<NonEmpty<BTreeSet<String>>>(r#"[]"#).is_err());
        assert!(serde_json::from_str::<NonEmpty<BTreeSet<String>>>(r#"["a", "a"]"#).is_err());
        assert!(serde_json::from_str::<NonEmpty<BTreeSet<NonEmpty<String>>>>(r#"[""]"#).is_err());
        assert!(
            serde_json::from_str::<NonEmpty<BTreeSet<NonEmpty<String>>>>(r#"["a", "a"]"#).is_err()
        );
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
