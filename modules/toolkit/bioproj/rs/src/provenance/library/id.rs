use super::p5p7;
use crate::UntypedId;
use serde::{Serialize, Serializer};

/// The owned identifier of any concrete library.
///
/// This resolved union serializes as a bare [`UntypedId`] but cannot deserialize
/// alone because the wire value omits its variant. Provenance must resolve it
/// during deserialization. Tagging the reference and implementing `Deserialize`
/// would introduce a duplicate variant tag that could disagree with the
/// referenced library.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LibraryId {
    /// A P5/P7 library.
    P5P7(p5p7::LibraryId),
}

impl LibraryId {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(&self) -> &UntypedId {
        match self {
            Self::P5P7(id) => id.as_untyped(),
        }
    }
}

impl Serialize for LibraryId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_untyped().serialize(serializer)
    }
}

impl From<p5p7::LibraryId> for LibraryId {
    fn from(id: p5p7::LibraryId) -> Self {
        Self::P5P7(id)
    }
}

/// A borrowed identifier of any concrete library.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LibraryIdRef<'a> {
    /// A P5/P7 library.
    P5P7(&'a p5p7::LibraryId),
}

impl<'a> LibraryIdRef<'a> {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(self) -> &'a UntypedId {
        match self {
            Self::P5P7(id) => id.as_untyped(),
        }
    }

    /// Clones this borrowed identifier into its owned union.
    pub fn to_owned(self) -> LibraryId {
        match self {
            Self::P5P7(id) => LibraryId::P5P7(id.clone()),
        }
    }
}
