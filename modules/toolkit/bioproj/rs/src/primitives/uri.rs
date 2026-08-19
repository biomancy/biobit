use super::NonEmpty;
use eyre::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

/// A workspace-local file locator.
///
/// This initial representation accepts only `file:<filename>`, where the
/// filename identifies one normal file directly beside `bioproj.toml`.
/// Resolution against that manifest directory is performed by the manifest
/// loader. General URI schemes and nested paths are intentionally deferred
/// until this primitive is backed by a dedicated URI crate.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uri(NonEmpty<String>);

impl Uri {
    /// Creates a validated workspace-local file locator.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let filename = value
            .strip_prefix("file:")
            .ok_or_else(|| eyre::eyre!("URI '{value}' must use the file: scheme"))?;

        ensure!(!filename.is_empty(), "URI '{value}' must name a file");
        ensure!(
            !filename.chars().any(char::is_whitespace),
            "URI '{value}' must not contain whitespace"
        );
        ensure!(
            !filename.contains(['/', '\\']),
            "URI '{value}' must name a file in the manifest directory"
        );
        ensure!(
            !filename.contains(['?', '#']),
            "URI '{value}' must not contain a query or fragment"
        );
        ensure!(
            matches!(
                Path::new(filename).components().next(),
                Some(Component::Normal(_))
            ) && Path::new(filename).components().count() == 1,
            "URI '{value}' must name one normal file in the manifest directory"
        );

        Ok(Self(NonEmpty::new(value)?))
    }

    /// Returns this locator in its serialized `file:<filename>` form.
    pub fn as_str(&self) -> &str {
        self.0.as_ref().as_str()
    }

    /// Resolves this locator against the containing manifest directory.
    pub(crate) fn resolve_against(&self, directory: &Path) -> PathBuf {
        // Construction ensures this slice is always present and is exactly one
        // normal filename.
        directory.join(&self.as_str()["file:".len()..])
    }

    /// Consumes this locator and returns its serialized form.
    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

impl Borrow<str> for Uri {
    fn borrow(&self) -> &str {
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

#[cfg(test)]
mod tests {
    use super::Uri;
    use std::collections::BTreeSet;
    use std::path::Path;

    #[test]
    fn accepts_a_file_in_the_manifest_directory() {
        let uri = Uri::new("file:provenance.json").unwrap();
        assert_eq!(uri.as_str(), "file:provenance.json");
        assert_eq!(
            uri.resolve_against(Path::new("workspace")),
            Path::new("workspace/provenance.json")
        );
    }

    #[test]
    fn supports_string_slice_set_lookups() {
        let locations = BTreeSet::from([Uri::new("file:provenance.json").unwrap()]);

        assert!(locations.contains("file:provenance.json"));
    }

    #[test]
    fn rejects_locations_outside_the_manifest_directory() {
        for value in [
            "",
            "provenance.json",
            "s3://bucket/provenance.json",
            "file:",
            "file://provenance.json",
            "file:/provenance.json",
            "file:./provenance.json",
            "file:../provenance.json",
            "file:payload/provenance.json",
            "file:provenance.json?version=1",
        ] {
            assert!(Uri::new(value).is_err(), "{value:?} should be rejected");
        }
    }
}
