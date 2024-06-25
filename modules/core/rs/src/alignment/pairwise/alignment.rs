// use std::ops::Range;
//
// use super::offset::OffsetSize;
// use super::step::{Step, StepSize};
// use super::utils;
//
// /// Pairwise alignment score - any integer type.
// pub trait Score: num::PrimInt {}
//
// impl<T: num::PrimInt> Score for T {}
//
// pub trait Alignment {
//     type StepSizeType: StepSize;
//
//
//
// }


// /// A genomic alignment between two sequences.
// pub struct Alignment<ScoreType = i64, StepSizeType = u8, Seq1OffsetType = u64, Seq2OffsetType = u64>
//     where
//         ScoreType: Score,
//         StepSizeType: StepSize,
//         Seq1OffsetType: OffsetSize,
//         Seq2OffsetType: OffsetSize
// {
//     pub score: ScoreType,
//     pub steps: Vec<Step<StepSizeType>>,
//     pub seq1: Range<Seq1OffsetType>,
//     pub seq2: Range<Seq2OffsetType>,
// }

// impl<
//     ScoreType: Score,
//     StepSizeType: StepSize,
//     Seq1OffsetType: OffsetSize,
//     Seq2OffsetType: OffsetSize
// > Alignment<ScoreType, StepSizeType, Seq1OffsetType, Seq2OffsetType> {
//     /// Checks if the alignment is empty.
//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
//
//     /// Returns the total length of the alignment - the sum of all step lengths.
//     pub fn len(&self) -> usize {
//         self.steps.iter().map(|x| x.len as usize).sum()
//     }
//
//     /// Returns the RLE representation of the alignment.
//     pub fn rle(&self) -> String {
//         utils::rle(&self.steps, self.len())
//     }
//
//     // pub fn prettify(&self, seq1: &str, seq2: &str) -> String {
//     //     let seq1 = &seq1[self.seq1.start..self.seq1.end];
//     //     let seq2 = &seq2[self.seq2.start..self.seq2.end];
//     //     let total: usize = self.len();
//     //     utils::prettify(seq1, seq2, &self.steps, total)
//     // }
//
//     // pub fn intersects(&self, other: &Alignment<S>) -> bool {
//     //     if max(self.seq1.start, other.seq1.start) >= min(self.seq1.end, other.seq1.end) {
//     //         return false;
//     //     }
//     //     if max(self.seq2.start, other.seq2.start) >= min(self.seq2.end, other.seq2.end) {
//     //         return false;
//     //     }
//     //     return utils::intersects(self.coalesced_steps(), other.coalesced_steps());
//     // }
//
//     // pub fn coalesced_steps(&self) -> impl Iterator<Item=CoalescedStep> + '_ {
//     //     CoalescedStepIter {
//     //         iter: self.steps.iter().peekable(),
//     //         offset: Offset {
//     //             seq1: self.seq1.start,
//     //             seq2: self.seq2.start,
//     //         },
//     //     }
//     // }
// }
