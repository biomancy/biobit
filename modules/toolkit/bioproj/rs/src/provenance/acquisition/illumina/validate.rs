use crate::provenance::Provenance;
use crate::provenance::library::p5p7;
use eyre::{Result, bail};

pub(super) fn validate<'a>(
    library_ids: impl IntoIterator<Item = &'a p5p7::LibraryId>,
    provenance: &Provenance,
) -> Result<()> {
    for library_id in library_ids {
        match provenance.get(library_id) {
            Some(Ok(_)) => {}
            Some(Err(error)) => return Err(error),
            None => bail!("Acquisition references unknown P5/P7 Library '{library_id}'"),
        }
    }
    Ok(())
}
