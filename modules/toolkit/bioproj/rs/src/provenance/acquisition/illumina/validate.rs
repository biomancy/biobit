use crate::provenance::library::p5p7;
use crate::provenance::{LibraryIdRef, Provenance};
use crate::{Error, Result, UntypedId};

pub(super) fn validate_against<'a>(
    acquisition_id: &UntypedId,
    library_ids: impl IntoIterator<Item = &'a p5p7::LibraryId>,
    provenance: &Provenance,
) -> Result<()> {
    for library_id in library_ids {
        // This checks that all library IDs are present in the provenance store
        // and that their typed id matches the expected p5/p7 library type.
        let key: LibraryIdRef<'_> = library_id.into();
        match provenance.get(&key) {
            Some(Ok(_)) => {}
            Some(Err(error)) => return Err(error),
            None => {
                return Err(Error::missing(
                    "Acquisition",
                    acquisition_id,
                    "libraries",
                    "Library",
                    library_id.as_untyped(),
                ));
            }
        }
    }
    Ok(())
}
