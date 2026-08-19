use super::NonEmpty;
use eyre::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// An untyped workspace-local identifier shared by entity-specific ID types.
///
/// IDs must match `[A-Za-z0-9_-]+` so they are safe to use in downstream
/// table-oriented environments without escaping.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct UntypedId(NonEmpty<String>);

impl UntypedId {
    /// Creates a validated identifier.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
            "ID '{value}' must match [A-Za-z0-9_-]+"
        );
        NonEmpty::new(value).map(Self)
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Consumes this untyped ID and returns its underlying string.
    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

impl AsRef<str> for UntypedId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for UntypedId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Display for UntypedId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<UntypedId> for String {
    fn from(value: UntypedId) -> Self {
        value.into_inner()
    }
}

impl FromStr for UntypedId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<String> for UntypedId {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for UntypedId {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for UntypedId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

macro_rules! define_entity_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name($crate::UntypedId);

        impl $name {
            /// Creates a validated entity-specific identifier.
            pub fn new(value: impl Into<::std::string::String>) -> ::eyre::Result<Self> {
                $crate::UntypedId::new(value).map(Self)
            }

            /// Returns the untyped workspace-local ID.
            pub fn as_untyped(&self) -> &$crate::UntypedId {
                &self.0
            }

            /// Consumes this typed ID and returns its untyped workspace-local ID.
            pub fn into_untyped(self) -> $crate::UntypedId {
                self.0
            }
        }

        impl ::std::borrow::Borrow<$crate::UntypedId> for $name {
            fn borrow(&self) -> &$crate::UntypedId {
                self.as_untyped()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, formatter)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::eyre::Report;

            fn from_str(value: &str) -> ::eyre::Result<Self> {
                Self::new(value)
            }
        }

        impl ::core::convert::TryFrom<::std::string::String> for $name {
            type Error = ::eyre::Report;

            fn try_from(value: ::std::string::String) -> ::eyre::Result<Self> {
                Self::new(value)
            }
        }

        impl ::core::convert::TryFrom<&str> for $name {
            type Error = ::eyre::Report;

            fn try_from(value: &str) -> ::eyre::Result<Self> {
                Self::new(value)
            }
        }
    };
}

pub(crate) use define_entity_id;

#[cfg(test)]
mod tests {
    use super::UntypedId;
    use crate::provenance::SourceId;
    use std::collections::BTreeMap;

    #[test]
    fn accepts_documented_identifier_charset() {
        for value in ["a", "SRC_01", "library-2", "A0_b-C"] {
            assert_eq!(UntypedId::new(value).unwrap().as_str(), value);
        }
    }

    #[test]
    fn rejects_invalid_identifiers() {
        for value in ["", "with space", "with.dot", "with/slash", "ünicode"] {
            assert!(UntypedId::new(value).is_err(), "{value:?} should be rejected");
        }
    }

    #[test]
    fn deserialization_applies_identifier_validation() {
        assert!(serde_json::from_str::<UntypedId>(r#""with space""#).is_err());
        assert!(serde_json::from_str::<SourceId>(r#""with space""#).is_err());
    }

    #[test]
    fn typed_ids_borrow_as_untyped_ids() {
        let typed = SourceId::new("ENTITY1").unwrap();
        let untyped = UntypedId::new("ENTITY1").unwrap();
        let values = BTreeMap::from([(typed, true)]);

        assert_eq!(values.get(&untyped), Some(&true));
    }
}
