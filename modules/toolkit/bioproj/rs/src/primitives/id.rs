use eyre::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A workspace-local identifier shared by all entity-specific ID types.
///
/// IDs must match `[A-Za-z0-9_-]+` so they are safe to use in downstream
/// table-oriented environments without escaping.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(String);

impl Id {
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
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes this ID and returns its underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<Id> for Id {
    fn as_ref(&self) -> &Id {
        self
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.into_inner()
    }
}

impl FromStr for Id {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<String> for Id {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Id {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
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
        pub struct $name($crate::Id);

        impl $name {
            /// Creates a validated entity-specific identifier.
            pub fn new(value: impl Into<::std::string::String>) -> ::eyre::Result<Self> {
                Ok(Self($crate::Id::new(value)?))
            }

            /// Returns the common workspace-local ID.
            pub fn as_id(&self) -> &$crate::Id {
                &self.0
            }

            /// Returns the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Consumes this typed ID and returns its common ID.
            pub fn into_id(self) -> $crate::Id {
                self.0
            }
        }

        impl ::core::convert::AsRef<$crate::Id> for $name {
            fn as_ref(&self) -> &$crate::Id {
                self.as_id()
            }
        }

        impl ::core::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
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
    use super::Id;

    #[test]
    fn accepts_documented_identifier_charset() {
        for value in ["a", "SRC_01", "library-2", "A0_b-C"] {
            assert_eq!(Id::new(value).unwrap().as_str(), value);
        }
    }

    #[test]
    fn rejects_invalid_identifiers() {
        for value in ["", "with space", "with.dot", "with/slash", "ünicode"] {
            assert!(Id::new(value).is_err(), "{value:?} should be rejected");
        }
    }
}
