use crate::Id;
use eyre::{Result, bail};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeSet;

/// Validates that all IDs in a scope are unique.
pub(crate) fn unique_ids<T>(scope: &str, ids: impl IntoIterator<Item = T>) -> Result<()>
where
    T: AsRef<Id>,
{
    let mut seen = BTreeSet::new();
    for value in ids {
        let id = value.as_ref();
        if !seen.insert(id.clone()) {
            bail!("ID '{id}' is not unique within {scope}");
        }
    }
    Ok(())
}

/// Validates and collects a non-empty set of non-empty strings.
pub(crate) fn non_empty_string_set(
    field: &str,
    values: impl IntoIterator<Item = impl Into<String>>,
) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for value in values {
        let value = value.into();
        if value.is_empty() {
            bail!("{field} must not contain empty strings");
        }
        if !result.insert(value.clone()) {
            bail!("{field} must not contain duplicate value '{value}'");
        }
    }
    if result.is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(result)
}

/// Deserializes a non-empty, duplicate-free set of non-empty strings.
pub(crate) fn deserialize_non_empty_string_set<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<String>::deserialize(deserializer)?;
    non_empty_string_set("strings", values).map_err(serde::de::Error::custom)
}

/// Deserializes a required, non-empty string.
pub(crate) fn deserialize_non_empty_string<'de, D>(
    deserializer: D,
) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    non_empty_string("string", value).map_err(serde::de::Error::custom)
}

/// Validates a required, non-empty string.
pub(crate) fn non_empty_string(field: &str, value: impl Into<String>) -> Result<String> {
    let value = value.into();
    if value.is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(value)
}
