use super::fastq;
use crate::UntypedId;
use serde::{Serialize, Serializer};

/// The owned identifier of any concrete dataset.
///
/// This resolved union serializes as a bare [`UntypedId`] but cannot deserialize
/// alone because the wire value omits its variant. The owning domain must
/// resolve it during deserialization. Tagging the reference and implementing
/// `Deserialize` would introduce a duplicate variant tag that could disagree
/// with the referenced dataset.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DatasetId {
    /// A collection of independent FASTQ files.
    Fastq(fastq::SingleId),
    /// Ordered pairs of FASTQ files.
    PairedFastq(fastq::PairedId),
}

impl DatasetId {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(&self) -> &UntypedId {
        match self {
            Self::Fastq(id) => id.as_untyped(),
            Self::PairedFastq(id) => id.as_untyped(),
        }
    }
}

impl Serialize for DatasetId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_untyped().serialize(serializer)
    }
}

impl From<fastq::SingleId> for DatasetId {
    fn from(id: fastq::SingleId) -> Self {
        Self::Fastq(id)
    }
}

impl From<fastq::PairedId> for DatasetId {
    fn from(id: fastq::PairedId) -> Self {
        Self::PairedFastq(id)
    }
}

/// A borrowed identifier of any concrete dataset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetIdRef<'a> {
    /// A collection of independent FASTQ files.
    Fastq(&'a fastq::SingleId),
    /// Ordered pairs of FASTQ files.
    PairedFastq(&'a fastq::PairedId),
}

impl<'a> DatasetIdRef<'a> {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(self) -> &'a UntypedId {
        match self {
            Self::Fastq(id) => id.as_untyped(),
            Self::PairedFastq(id) => id.as_untyped(),
        }
    }

    /// Clones this borrowed identifier into its owned union.
    pub fn to_owned(self) -> DatasetId {
        match self {
            Self::Fastq(id) => DatasetId::Fastq(id.clone()),
            Self::PairedFastq(id) => DatasetId::PairedFastq(id.clone()),
        }
    }
}
