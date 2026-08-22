use super::illumina;
use crate::UntypedId;
use serde::{Serialize, Serializer};

/// The owned identifier of any concrete acquisition.
///
/// Concrete IDs remain distinct at compatibility boundaries. This closed
/// union is used only by heterogeneous relationships such as design units and
/// datasets.
///
/// This resolved union serializes as a bare [`UntypedId`] but cannot deserialize
/// alone because the wire value omits its variant. The owning domain must
/// resolve it during deserialization. Tagging the reference and implementing
/// `Deserialize` would introduce a duplicate variant tag that could disagree
/// with the referenced acquisition.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AcquisitionId {
    /// A single-end Illumina sequencing acquisition.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencingId),
    /// A paired-end Illumina sequencing acquisition.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencingId),
}

impl AcquisitionId {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(&self) -> &UntypedId {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.as_untyped(),
            Self::IlluminaPairedEndSequencing(id) => id.as_untyped(),
        }
    }
}

impl Serialize for AcquisitionId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_untyped().serialize(serializer)
    }
}

impl From<illumina::SingleEndSequencingId> for AcquisitionId {
    fn from(id: illumina::SingleEndSequencingId) -> Self {
        Self::IlluminaSingleEndSequencing(id)
    }
}

impl From<illumina::PairedEndSequencingId> for AcquisitionId {
    fn from(id: illumina::PairedEndSequencingId) -> Self {
        Self::IlluminaPairedEndSequencing(id)
    }
}

/// A borrowed identifier of any concrete acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcquisitionIdRef<'a> {
    /// A single-end Illumina sequencing acquisition.
    IlluminaSingleEndSequencing(&'a illumina::SingleEndSequencingId),
    /// A paired-end Illumina sequencing acquisition.
    IlluminaPairedEndSequencing(&'a illumina::PairedEndSequencingId),
}

impl<'a> AcquisitionIdRef<'a> {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(self) -> &'a UntypedId {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.as_untyped(),
            Self::IlluminaPairedEndSequencing(id) => id.as_untyped(),
        }
    }

    /// Clones this borrowed identifier into its owned union.
    pub fn to_owned(self) -> AcquisitionId {
        match self {
            Self::IlluminaSingleEndSequencing(id) => {
                AcquisitionId::IlluminaSingleEndSequencing(id.clone())
            }
            Self::IlluminaPairedEndSequencing(id) => {
                AcquisitionId::IlluminaPairedEndSequencing(id.clone())
            }
        }
    }
}
