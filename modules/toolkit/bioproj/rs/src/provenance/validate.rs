use super::{Provenance, acquisition, library, sample};
use crate::UntypedId;
use eyre::{Result, bail};
use std::cmp::Ordering;

pub(super) fn validate(provenance: &Provenance) -> Result<()> {
    validate_disjoint_ids(provenance)?;
    sample::validate(provenance)?;
    library::validate(provenance)?;
    acquisition::validate(provenance)
}

fn validate_disjoint_ids(provenance: &Provenance) -> Result<()> {
    disjoint(
        provenance.sources.keys().map(|id| id.as_untyped()),
        provenance.samples.keys().map(|id| id.as_untyped()),
    )?;
    disjoint(
        provenance.sources.keys().map(|id| id.as_untyped()),
        provenance.libraries.keys(),
    )?;
    disjoint(
        provenance.sources.keys().map(|id| id.as_untyped()),
        provenance.acquisitions.keys(),
    )?;
    disjoint(
        provenance.samples.keys().map(|id| id.as_untyped()),
        provenance.libraries.keys(),
    )?;
    disjoint(
        provenance.samples.keys().map(|id| id.as_untyped()),
        provenance.acquisitions.keys(),
    )?;
    disjoint(provenance.libraries.keys(), provenance.acquisitions.keys())
}

/// Checks two ID iterators that are ordered by their untyped values.
fn disjoint<'a, 'b>(
    left: impl IntoIterator<Item = &'a UntypedId>,
    right: impl IntoIterator<Item = &'b UntypedId>,
) -> Result<()> {
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();

    while let (Some(left_id), Some(right_id)) = (left.peek(), right.peek()) {
        match left_id.cmp(right_id) {
            Ordering::Less => {
                left.next();
            }
            Ordering::Greater => {
                right.next();
            }
            Ordering::Equal => {
                bail!("ID '{left_id}' is not unique within provenance");
            }
        }
    }
    Ok(())
}
