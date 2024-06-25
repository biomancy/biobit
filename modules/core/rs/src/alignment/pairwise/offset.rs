use crate::num::PrimUInt;

/// Offset of the alignment in sequence coordinates
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Offset<Seq1Offset: PrimUInt = u64, Seq2Offset: PrimUInt = u64> {
    pub seq1: Seq1Offset,
    pub seq2: Seq2Offset,
}
