use std::cmp::{max, min};
use std::fmt::Display;
use std::ops::Range;

use derive_getters::{Dissolve, Getters};
use derive_more::{Constructor, From, Into};

use biobit_core_rs::num::{Num, PrimUInt};

use super::step::{Step, StepWithOffset, StepsWithOffsetsIterator};
use super::utils;

/// A genomic alignment between two sequences.
#[derive(Clone, Eq, PartialEq, Debug, Getters, Constructor, Dissolve, From, Into)]
pub struct Alignment<Score, StepLen, Seq1Idx, Seq2Idx>
where
    Score: Num,
    StepLen: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
{
    score: Score,
    steps: Vec<Step<StepLen>>,
    seq1: Range<Seq1Idx>,
    seq2: Range<Seq2Idx>,
}

impl<Score, StepLen, Seq1Idx, Seq2Idx> Alignment<Score, StepLen, Seq1Idx, Seq2Idx>
where
    Score: Num,
    StepLen: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
{
    /// Add a step to the alignment.
    pub fn push(&mut self, step: Step<StepLen>) {
        self.steps.push(step);
    }

    /// Checks if the alignment is empty.
    pub fn is_empty(&self) -> bool {
        // Empty alignment is an alignment with no steps.
        // Note: length of each step is guaranteed to be non-zero.
        self.steps.is_empty()
    }

    /// Returns the total length of the alignment - the sum of all step lengths.
    pub fn len<Acc: PrimUInt + From<StepLen>>(&self) -> Acc {
        let mut total = Acc::zero();
        for step in &self.steps {
            total = <Acc as From<StepLen>>::from(*step.len());
        }
        total
    }

    /// Returns the RLE representation of the alignment.
    pub fn rle(&self) -> String
    where
        StepLen: Display,
    {
        Step::rle_string(self.steps.iter())
    }

    /// Returns true if the alignment intersects with another alignment.
    pub fn intersects(&self, other: &Self) -> bool
    where
        usize: From<Seq1Idx> + From<Seq2Idx> + From<StepLen>,
    {
        // Fast check if the ranges do not overlap
        if max(self.seq1.start, other.seq1.start) >= min(self.seq1.end, other.seq1.end) {
            return false;
        }
        if max(self.seq2.start, other.seq2.start) >= min(self.seq2.end, other.seq2.end) {
            return false;
        }
        return utils::intersects::<_, _, _, usize>(self.tracked_steps(), other.tracked_steps());
    }

    /// Returns alignment steps with tracked sequence coordinates.
    pub fn tracked_steps(
        &self,
    ) -> impl Iterator<Item = StepWithOffset<StepLen, Seq1Idx, Seq2Idx>> + '_ {
        StepsWithOffsetsIterator::new(
            self.steps.iter().cloned(),
            (self.seq1.start, self.seq2.start).into(),
        )
    }
}
