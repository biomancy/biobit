use super::Acquisition;
use crate::provenance::Provenance;
use eyre::Result;

/// Validates every Acquisition-to-Library edge in a provenance graph.
pub(crate) fn validate(provenance: &Provenance) -> Result<()> {
    for acquisition in provenance.acquisitions() {
        match acquisition {
            Acquisition::IlluminaSingleEndSequencing(acquisition) => {
                acquisition.validate(provenance)?;
            }
            Acquisition::IlluminaPairedEndSequencing(acquisition) => {
                acquisition.validate(provenance)?;
            }
        }
    }
    Ok(())
}
