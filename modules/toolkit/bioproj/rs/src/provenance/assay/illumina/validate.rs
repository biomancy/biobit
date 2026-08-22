use crate::UntypedId;
use crate::provenance::library::Library;
use crate::provenance::library::p5p7::LibraryId;
use eyre::{Result, bail};
use std::collections::BTreeMap;

pub(super) fn validate(
    library_id: &LibraryId,
    libraries: &BTreeMap<UntypedId, Library>,
) -> Result<()> {
    match libraries.get(library_id.as_untyped()) {
        Some(Library::P5P7(_)) => Ok(()),
        None => bail!("Assay references unknown P5/P7 Library '{library_id}'"),
    }
}
