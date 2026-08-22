use super::fastq;
use crate::UntypedId;
use serde::{Serialize, Serializer};

/// The owned identifier of any concrete asset.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AssetId {
    /// A FASTQ file.
    Fastq(fastq::FastqId),
}

impl AssetId {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(&self) -> &UntypedId {
        match self {
            Self::Fastq(id) => id.as_untyped(),
        }
    }
}

impl Serialize for AssetId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_untyped().serialize(serializer)
    }
}

impl From<fastq::FastqId> for AssetId {
    fn from(id: fastq::FastqId) -> Self {
        Self::Fastq(id)
    }
}

/// A borrowed identifier of any concrete asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetIdRef<'a> {
    /// A FASTQ file.
    Fastq(&'a fastq::FastqId),
}

impl<'a> AssetIdRef<'a> {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(self) -> &'a UntypedId {
        match self {
            Self::Fastq(id) => id.as_untyped(),
        }
    }

    /// Clones this borrowed identifier into its owned union.
    pub fn to_owned(self) -> AssetId {
        match self {
            Self::Fastq(id) => AssetId::Fastq(id.clone()),
        }
    }
}
