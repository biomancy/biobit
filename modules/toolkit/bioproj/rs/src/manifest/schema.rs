use eyre::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The version of the manifest schema.
///
/// Schema versions are serialized as canonical `major.minor.patch` strings.
/// They select a manifest layout and adapter orchestration strategy; they do
/// not belong to the resolved [`crate::Project`] graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Schema {
    /// The breaking schema version component.
    pub major: u32,
    /// The backwards-compatible schema version component.
    pub minor: u32,
    /// The backwards-compatible schema version component.
    pub patch: u32,
}

impl Schema {
    /// The only schema currently supported by the manifest loader.
    pub const CURRENT: Self = Self::new(0, 0, 1);

    /// Creates a schema version from its numeric components.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub(crate) fn ensure_supported(self) -> Result<()> {
        if self != Self::CURRENT {
            bail!(
                "unsupported bioproj schema '{self}'; this loader supports '{}'",
                Self::CURRENT
            );
        }
        Ok(())
    }
}

impl Display for Schema {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Schema {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let mut components = value.split('.');
        let major = parse_component("major", components.next(), value)?;
        let minor = parse_component("minor", components.next(), value)?;
        let patch = parse_component("patch", components.next(), value)?;
        if components.next().is_some() {
            bail!("schema '{value}' must have exactly major.minor.patch components");
        }
        Ok(Self::new(major, minor, patch))
    }
}

fn parse_component(component: &str, value: Option<&str>, schema: &str) -> Result<u32> {
    let value = value.ok_or_else(|| {
        eyre::eyre!("schema '{schema}' must have exactly major.minor.patch components")
    })?;
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        bail!("schema '{schema}' has an invalid {component} component");
    }
    value
        .parse()
        .map_err(|_| eyre::eyre!("schema '{schema}' has an out-of-range {component} component"))
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
