use crate::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The version of a stored project document.
///
/// Schema versions are serialized as canonical `major.minor.patch` strings.
/// They select a versioned representation and do not belong to the resolved
/// [`crate::Project`] graph or identify a scientific release.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Schema {
    major: u16,
    minor: u16,
    patch: u16,
}

impl Schema {
    /// The only schema currently supported by I/O adapters.
    pub const CURRENT: Self = Self::new(0, 0, 1);

    /// Creates a schema version from its numeric components.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns the major version component.
    pub const fn major(&self) -> u16 {
        self.major
    }

    /// Returns the minor version component.
    pub const fn minor(&self) -> u16 {
        self.minor
    }

    /// Returns the patch version component.
    pub const fn patch(&self) -> u16 {
        self.patch
    }
}

impl Display for Schema {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Schema {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self> {
        let mut components = value.split('.');
        let major = parse_component("major", components.next(), value)?;
        let minor = parse_component("minor", components.next(), value)?;
        let patch = parse_component("patch", components.next(), value)?;
        if components.next().is_some() {
            return Err(Error::invalid(
                "schema",
                format!("'{value}' must have exactly major.minor.patch components"),
            ));
        }
        Ok(Self::new(major, minor, patch))
    }
}

fn parse_component(component: &str, value: Option<&str>, schema: &str) -> Result<u16> {
    let value = value.ok_or_else(|| {
        Error::invalid(
            "schema",
            format!("'{schema}' must have exactly major.minor.patch components"),
        )
    })?;
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(Error::invalid(
            "schema",
            format!("'{schema}' has an invalid {component} component"),
        ));
    }
    value.parse().map_err(|_| {
        Error::invalid(
            "schema",
            format!("'{schema}' has an out-of-range {component} component"),
        )
    })
}

impl Serialize for Schema {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Schema;
    use std::str::FromStr;

    #[test]
    fn parses_and_serializes_a_schema() {
        let schema = Schema::from_str("0.0.1").unwrap();
        assert_eq!(schema, Schema::CURRENT);
        assert_eq!((schema.major(), schema.minor(), schema.patch()), (0, 0, 1));
        assert_eq!(serde_json::to_string(&schema).unwrap(), r#""0.0.1""#);
    }

    #[test]
    fn rejects_noncanonical_schema_strings() {
        for value in ["", "0", "0.0", "0.0.1.0", "v0.0.1", "00.0.1", "0.-1.1"] {
            assert!(
                Schema::from_str(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }
}
