use crate::provenance::Provenance;
use eyre::{Result, bail};

/// Validates every Library-to-Sample edge in a provenance graph.
pub(crate) fn validate(provenance: &Provenance) -> Result<()> {
    for library in provenance.libraries() {
        for sample_id in library.samples() {
            if provenance.get(sample_id).is_none() {
                bail!(
                    "Library '{}' references unknown Sample '{sample_id}'",
                    library.id().as_untyped()
                );
            }
        }
    }
    Ok(())
}
