use super::illumina;
use crate::primitives::define_entity_family;

define_entity_family! {
    /// A concrete logical acquisition.
    ///
    /// This initial type intentionally combines two concepts: the reusable assay
    /// specification and one execution of that assay. A future model may separate
    /// them so repeated acquisitions can reference one shared method definition.
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

#[cfg(test)]
mod tests {
    use super::{Acquisition, AcquisitionId, AcquisitionKind};
    use crate::provenance::acquisition::illumina::{
        PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
    };
    use crate::provenance::library::p5p7;
    use kinded::Kinded as _;

    #[test]
    fn distinguishes_kind_from_strict_identity_matching() {
        let id = SingleEndSequencingId::new("ACQ1").unwrap();
        let acquisition: Acquisition = SingleEndSequencing::new(
            id.clone(),
            [p5p7::LibraryId::new("LIB1").unwrap()],
            Default::default(),
            None::<String>,
        )
        .unwrap()
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
        assert!(matching.matches(&acquisition));
        assert!(!same_kind.matches(&acquisition));
        assert!(!same_value.matches(&acquisition));
    }
}
