use super::fastq;
use crate::primitives::define_entity_family;

define_entity_family! {
    /// A concrete immutable data artifact.
    ///
    /// Assets describe storage only. Acquisition membership and file layout are
    /// declared by [`crate::data::Dataset`] records.
    pub family Asset {
        kind: AssetKind,
        id: AssetId,
        id_ref: AssetIdRef,
        kind_doc: "The concrete type of an asset or asset identifier.",
        variants: {
            /// One FASTQ file.
            Fastq(fastq::Fastq, fastq::FastqId),
        }
    }
}
