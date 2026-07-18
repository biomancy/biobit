use eyre::{Result, bail, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// An absolute URI that locates an asset representation.
///
/// The value is preserved verbatim. Validation is deliberately limited to the
/// URI scheme and the absence of whitespace, so custom repository schemes such
/// as `s3`, `drs`, and future archive schemes remain usable.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uri(String);

impl Uri {
    /// Creates a validated absolute URI.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(!value.is_empty(), "URI must not be empty");
        ensure!(
            !value.chars().any(char::is_whitespace),
            "URI '{value}' must not contain whitespace"
        );

        let (scheme, remainder) = value
            .split_once(':')
            .ok_or_else(|| eyre::eyre!("URI '{value}' must have an absolute scheme"))?;
        ensure!(
            !scheme.is_empty(),
            "URI '{value}' must have an absolute scheme"
        );
        ensure!(
            scheme
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic())
                && scheme
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-')),
            "URI '{value}' has an invalid scheme"
        );
        ensure!(
            !remainder.is_empty(),
            "URI '{value}' must have a value after its scheme"
        );

        Ok(Self(value))
    }

    /// Returns this URI as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes this URI and returns its underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for Uri {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Uri {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<Uri> for String {
    fn from(value: Uri) -> Self {
        value.into_inner()
    }
}

impl FromStr for Uri {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<String> for Uri {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Uri {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

pub(crate) fn non_empty_uri_set(
    field: &str,
    values: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<BTreeSet<Uri>> {
    let mut result = BTreeSet::new();
    for value in values {
        let uri = Uri::new(value.as_ref())?;
        if !result.insert(uri.clone()) {
            bail!("{field} must not contain duplicate URI '{uri}'");
        }
    }
    ensure!(!result.is_empty(), "{field} must not be empty");
    Ok(result)
}

pub(crate) fn deserialize_non_empty_uri_set<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeSet<Uri>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<Uri>::deserialize(deserializer)?;
    non_empty_uri_set("locations", values).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::Uri;

    #[test]
    fn accepts_custom_absolute_uri_schemes() {
        for uri in [
            "file:///project/read.fq.gz",
            "s3://bucket/read.fq.gz",
            "drs://archive/object",
        ] {
            assert_eq!(Uri::new(uri).unwrap().as_str(), uri);
        }
    }

    #[test]
    fn rejects_non_uri_locations() {
        for value in ["", "reads.fq.gz", "https://example.org/a file"] {
            assert!(Uri::new(value).is_err(), "{value:?} should be rejected");
        }
    }
}
