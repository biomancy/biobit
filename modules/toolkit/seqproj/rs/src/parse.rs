use eyre::Result;
use eyre::ensure;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

pub fn ind(ind: impl Into<String>) -> Result<String> {
    let ind = ind.into();
    ensure!(!ind.is_empty(), "ID must not be an empty string");
    Ok(ind)
}

pub fn meta(
    meta: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
) -> Result<BTreeMap<String, String>> {
    meta.into_iter()
        .map(|(key, value)| {
            let (key, value) = (key.into(), value.into());
            ensure!(!key.is_empty(), "Meta keys must not be an empty string");
            ensure!(!value.is_empty(), "Meta value must not be an empty string");
            Ok((key, value))
        })
        .collect()
}

pub fn non_empty_string(
    name: &str,
    description: Option<impl Into<String>>,
) -> Result<Option<String>> {
    let description = description.map(Into::into);
    if let Some(desc) = &description {
        ensure!(!desc.is_empty(), "{name} must not be an empty string");
    }
    Ok(description)
}

pub fn non_zero_u64(name: &str, value: Option<impl Into<u64>>) -> Result<Option<NonZeroU64>> {
    if let Some(v) = value {
        let non_zero =
            NonZeroU64::new(v.into()).ok_or_else(|| eyre::eyre!("{name} must be non-zero"))?;
        Ok(Some(non_zero))
    } else {
        Ok(None)
    }
}

pub fn set_of_non_empty_strings(
    name: &str,
    values: impl IntoIterator<Item = impl Into<String>>,
) -> Result<BTreeSet<String>> {
    let values: BTreeSet<String> = values
        .into_iter()
        .map(|value| {
            let value = value.into();
            ensure!(!value.is_empty(), "{name} must not contain empty strings");
            Ok(value)
        })
        .collect::<Result<_>>()?;
    ensure!(
        !values.is_empty(),
        "{name} must be specified, got empty collection"
    );
    Ok(values)
}
