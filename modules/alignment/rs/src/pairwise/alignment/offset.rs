use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};

use crate::pairwise::Step;
use biobit_core_rs::num::PrimUInt;

/// Offset of the alignment in sequence coordinates
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Debug,
    Hash,
    Default,
    Constructor,
    Dissolve,
    From,
    Into,
)]
pub struct Offset<Seq1Idx: PrimUInt, Seq2Idx: PrimUInt> {
    pub seq1: Seq1Idx,
    pub seq2: Seq2Idx,
}

impl<Seq1Idx: PrimUInt, Seq2Idx: PrimUInt> Offset<Seq1Idx, Seq2Idx> {
    pub fn apply<Len>(mut self, step: &Step<Len>) -> Self
    where
        Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    {
        step.op().apply(&mut self.seq1, &mut self.seq2, *step.len());
        self
    }
}
