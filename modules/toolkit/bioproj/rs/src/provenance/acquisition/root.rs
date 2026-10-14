use super::illumina;
use crate::primitives::define_entity_family;
use crate::provenance::{Provenance, library::p5p7};
use crate::{NonEmpty, OrderedSet, Result};

define_entity_family! {
    /// A concrete logical acquisition.
    ///
    /// One indivisible logical measurement, such as one demultiplexed sequencing
    /// outcome. Separately interpretable outputs have separate Acquisition IDs,
    /// even when measured in the same physical run. Batching and detailed
    /// protocols are outside this graph.
    pub family Acquisition {
        kind: AcquisitionKind,
        id: AcquisitionId,
        id_ref: AcquisitionIdRef,
        kind_doc: "The concrete type of an acquisition or acquisition identifier.",
        variants: {
            /// Single-end Illumina sequencing.
            IlluminaSingleEndSequencing(
                illumina::SingleEndSequencing,
                illumina::SingleEndSequencingId
            ),
            /// Paired-end Illumina sequencing.
            IlluminaPairedEndSequencing(
                illumina::PairedEndSequencing,
                illumina::PairedEndSequencingId
            ),
        }
    }
}

impl Acquisition {
    /// Returns the IDs of the libraries measured by this acquisition.
    pub fn libraries(&self) -> &NonEmpty<OrderedSet<p5p7::LibraryId>> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.libraries(),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.libraries(),
        }
    }

    /// Validates every Acquisition-to-Library edge in a provenance graph.
    pub(crate) fn validate_against(provenance: &Provenance) -> Result<()> {
        for acquisition in provenance.acquisitions() {
            match acquisition {
                Self::IlluminaSingleEndSequencing(acquisition) => {
                    acquisition.validate_against(provenance)?;
                }
                Self::IlluminaPairedEndSequencing(acquisition) => {
                    acquisition.validate_against(provenance)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Acquisition, AcquisitionId, AcquisitionKind};
    use crate::provenance::acquisition::illumina::{
        PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
    };
    use crate::provenance::library::p5p7;
    use kinded::Kinded as _;
    use std::collections::HashSet;

    #[test]
    fn family_ids_compare_with_borrowed_entity_ids() {
        let id = SingleEndSequencingId::new("ACQ1").unwrap();
        let acquisition: Acquisition = SingleEndSequencing::new(
            id.clone(),
            crate::NonEmpty::try_from_iter([p5p7::LibraryId::new("LIB1").unwrap()]).unwrap(),
            Default::default(),
            None,
        )
        .into();

        let matching = AcquisitionId::from(id);
        let same_kind = AcquisitionId::from(SingleEndSequencingId::new("ACQ2").unwrap());
        let same_value = AcquisitionId::from(PairedEndSequencingId::new("ACQ1").unwrap());

        assert_eq!(
            matching.kind(),
            AcquisitionKind::IlluminaSingleEndSequencing
        );
        assert_eq!(matching.kind(), same_kind.kind());
        assert_ne!(matching.kind(), same_value.kind());
        assert_eq!(matching, acquisition.id());
        assert_ne!(same_kind, acquisition.id());
        assert_ne!(same_value, acquisition.id());
    }

    #[test]
    fn family_id_identity_includes_its_kind() {
        let single = AcquisitionId::from(SingleEndSequencingId::new("ACQ1").unwrap());
        let paired = AcquisitionId::from(PairedEndSequencingId::new("ACQ1").unwrap());

        assert_eq!(single.as_untyped(), paired.as_untyped());
        assert_ne!(single, paired);
        assert_ne!(single.cmp(&paired), std::cmp::Ordering::Equal);
        assert_eq!(HashSet::from([single, paired]).len(), 2);
    }
}
