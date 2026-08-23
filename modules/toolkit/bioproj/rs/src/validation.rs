use crate::UntypedId;
use eyre::{Result, bail};
use std::collections::BTreeSet;

/// Validates that all IDs in a scope are unique.
pub(crate) fn unique_ids<'a>(
    scope: &str,
    ids: impl IntoIterator<Item = &'a UntypedId>,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            bail!("ID '{id}' is not unique within {scope}");
        }
    }
    Ok(())
}
