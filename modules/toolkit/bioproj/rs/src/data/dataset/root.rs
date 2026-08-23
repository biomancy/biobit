use super::fastq;
use crate::primitives::define_entity_family;
use crate::provenance::AcquisitionIdRef;

define_entity_family! {
    /// A complete stored form of one acquisition.
    ///
    /// Different datasets normally preserve the same acquisition data using
    /// different file layouts or encodings. Reusing assets across datasets is
    /// allowed, including for explicitly selected QC subsets.
    pub family Dataset {
        kind: DatasetKind,
        id: DatasetId,
        id_ref: DatasetIdRef,
        kind_doc: "The concrete type of a dataset or dataset identifier.",
        variants: {
            /// One or more independent FASTQ files.
            Fastq(fastq::Single, fastq::SingleId),
            /// One or more ordered read-one/read-two FASTQ pairs.
            PairedFastq(fastq::Paired, fastq::PairedId),
        }
    }
}

impl Dataset {
    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::Fastq(dataset) => dataset.acquisition().id(),
            Self::PairedFastq(dataset) => dataset.acquisition().id(),
        }
    }
}
