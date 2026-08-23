use super::{
    illumina,
    root::{Acquisition, AcquisitionKind},
};
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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, derive_more::From)]
pub enum AcquisitionId {
    /// A single-end Illumina sequencing acquisition.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencingId),
    /// A paired-end Illumina sequencing acquisition.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencingId),
}

impl AcquisitionId {
    /// Borrows this identifier as its concrete tagged reference.
    pub fn as_ref(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.into(),
            Self::IlluminaPairedEndSequencing(id) => id.into(),
        }
    }

    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(&self) -> &UntypedId {
        self.as_ref().as_untyped()
    }

    /// Returns whether this identifier has the same type and value as the
    /// acquisition's identifier.
    pub fn matches(&self, acquisition: &Acquisition) -> bool {
        self.as_ref().matches(acquisition)
    }
}

impl kinded::Kinded for AcquisitionId {
    type Kind = AcquisitionKind;

    fn kind(&self) -> Self::Kind {
        self.as_ref().kind()
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

/// A borrowed identifier of any concrete acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, derive_more::From)]
pub enum AcquisitionIdRef<'a> {
    /// A single-end Illumina sequencing acquisition.
    IlluminaSingleEndSequencing(&'a illumina::SingleEndSequencingId),
    /// A paired-end Illumina sequencing acquisition.
    IlluminaPairedEndSequencing(&'a illumina::PairedEndSequencingId),
}

impl<'a> AcquisitionIdRef<'a> {
    /// Returns whether this identifier has the same type and value as the
    /// acquisition's identifier.
    pub fn matches(self, acquisition: &Acquisition) -> bool {
        self == acquisition.id()
    }

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

impl kinded::Kinded for AcquisitionIdRef<'_> {
    type Kind = AcquisitionKind;

    fn kind(&self) -> Self::Kind {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.kind(),
            Self::IlluminaPairedEndSequencing(id) => id.kind(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AcquisitionId;
    use crate::provenance::acquisition::illumina::{
        PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
    };
    use crate::provenance::acquisition::{Acquisition, AcquisitionKind};
    use crate::provenance::library::p5p7;
    use kinded::Kinded as _;

    #[test]
    fn distinguishes_kind_from_strict_identity_matching() {
        let id = SingleEndSequencingId::new("ACQ1").unwrap();
        let acquisition = Acquisition::IlluminaSingleEndSequencing(
            SingleEndSequencing::new(
                id.clone(),
                [p5p7::LibraryId::new("LIB1").unwrap()],
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        );

        let matching = AcquisitionId::from(id);
        let same_kind = AcquisitionId::from(SingleEndSequencingId::new("ACQ2").unwrap());
        let same_value = AcquisitionId::from(PairedEndSequencingId::new("ACQ1").unwrap());

        assert_eq!(
            matching.kind(),
            AcquisitionKind::IlluminaSingleEndSequencing
        );
        assert_eq!(matching.kind(), same_kind.kind());
        assert_ne!(matching.kind(), same_value.kind());
        assert!(matching.matches(&acquisition));
        assert!(!same_kind.matches(&acquisition));
        assert!(!same_value.matches(&acquisition));
    }
}
