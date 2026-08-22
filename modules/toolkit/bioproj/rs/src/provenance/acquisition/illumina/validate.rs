use crate::UntypedId;
use crate::provenance::library::Library;
use crate::provenance::library::p5p7;
use eyre::{Result, bail};
use std::collections::BTreeMap;

pub(super) fn validate<'a>(
    library_ids: impl IntoIterator<Item = &'a p5p7::LibraryId>,
    libraries: &BTreeMap<UntypedId, Library>,
) -> Result<()> {
    for library_id in library_ids {
        match libraries.get(library_id.as_untyped()) {
            Some(Library::P5P7(_)) => {}
            None => bail!("Acquisition references unknown P5/P7 Library '{library_id}'"),
        }
    }
    Ok(())
}
