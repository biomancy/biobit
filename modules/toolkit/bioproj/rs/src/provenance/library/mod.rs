//! Concrete library preparations.

pub mod illumina;
pub mod strandedness;

use crate::Id;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::SampleId;

/// A concrete, sequencing-ready library preparation.
///
/// Each variant owns a fully specified prepared material type. Platform
/// modules contain the corresponding concrete struct and its properties. This
/// outer enum owns the serialized `type` discriminator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Library {
    /// An Illumina-compatible library prepared from DNA.
    IlluminaDna(illumina::DnaLibrary),
    /// An Illumina-compatible library prepared from RNA-derived cDNA.
    IlluminaCdna(illumina::CdnaLibrary),
}

impl Library {
    /// Returns the common workspace-local identifier for this library.
    pub fn id(&self) -> &Id {
        match self {
            Self::IlluminaDna(library) => library.id().as_id(),
            Self::IlluminaCdna(library) => library.id().as_id(),
        }
    }

    /// Returns the IDs of this library's parent samples.
    pub fn samples(&self) -> &BTreeSet<SampleId> {
        match self {
            Self::IlluminaDna(library) => library.samples(),
            Self::IlluminaCdna(library) => library.samples(),
        }
    }
}

impl AsRef<Id> for Library {
    fn as_ref(&self) -> &Id {
        self.id()
    }
}

#[cfg(test)]
mod tests {
    use super::Library;
    use crate::provenance::SampleId;
    use crate::provenance::library::illumina::{CdnaLibrary, CdnaLibraryId};
    use crate::provenance::library::strandedness::Strandedness;

    #[test]
    fn serializes_flat_tagged_library_variants() {
        let library = Library::IlluminaCdna(
            CdnaLibrary::new(
                CdnaLibraryId::new("LIB1").unwrap(),
                [SampleId::new("SMP1").unwrap()],
                ["cDNA"],
                ["poly-A"],
                Strandedness::Forward,
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap(),
        );

        assert_eq!(
            serde_json::to_string(&library).unwrap(),
            r#"{"type":"IlluminaCdna","id":"LIB1","samples":["SMP1"],"molecule":["cDNA"],"selection":["poly-A"],"strandedness":"Forward"}"#
        );
    }
}
