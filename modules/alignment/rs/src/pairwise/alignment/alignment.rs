use std::cmp::{max, min};
use std::fmt::Display;
use std::ops::Range;

use derive_getters::{Dissolve, Getters};
use derive_more::{Constructor, From, Into};

use biobit_core_rs::num::{Num, PrimUInt};

use super::step::{Step, StepWithOffset, StepsWithOffsetsIterator};
use super::utils;

/// A genomic alignment between two sequences.
/// The supplied ranges must match the sequence positions consumed by the steps.
#[derive(Clone, Eq, PartialEq, Debug, Getters, Constructor, Dissolve, From, Into)]
// derive_more::Constructor generates explicit field names.
#[allow(clippy::redundant_field_names)]
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
            total = total + <Acc as From<StepLen>>::from(*step.len());
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
        // Fast reject separated ranges here; the step traversal resolves touching ones.
        if max(self.seq1.start, other.seq1.start) > min(self.seq1.end, other.seq1.end) {
            return false;
        }
        if max(self.seq2.start, other.seq2.start) > min(self.seq2.end, other.seq2.end) {
            return false;
        }
        utils::intersects::<_, _, _, usize>(self.tracked_steps(), other.tracked_steps())
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

#[cfg(test)]
mod tests {
    use super::Alignment;
    use crate::pairwise::{Op, Step};
    use itertools::Itertools;
    use std::collections::BTreeSet;

    type Aln = Alignment<i32, u8, usize, usize>;
    type Points = BTreeSet<(usize, usize)>;

    fn alignment(steps: Vec<Step<u8>>, start: (usize, usize)) -> (Aln, Points) {
        let points = Op::unroll(&steps, start).collect();
        let mut end = start;
        for step in &steps {
            step.op().apply(&mut end.0, &mut end.1, *step.len());
        }

        (
            Alignment::new(0, steps, start.0..end.0, start.1..end.1),
            points,
        )
    }

    /// Compare an complete set of short paths against directly enumerated visited positions.
    /// An intersection is equivalent to a shared coordinate in the unrolled path.
    #[test]
    fn alignment_intersections_match_visited_positions_for_both_step_partitions() {
        let mut paths = Vec::new();
        let unit_steps =
            [Op::Match, Op::GapFirst, Op::GapSecond].map(|op| Step::new(op, 1u8).unwrap());

        // Cartesian powers generate every operation sequence of lengths 0 through 3.
        for len in 0..=3 {
            for steps in std::iter::repeat_n(unit_steps, len).multi_cartesian_product() {
                // Shift paths relative to each other along either coordinate axis.
                for start in [(0, 0), (0, 1), (1, 0)] {
                    let original = alignment(steps.clone(), start);
                    // Include both unit-step and merged-run representations so the
                    // intersection result cannot depend on how a path is partitioned.
                    let mut collapsed = steps.clone();
                    Step::collapse(&mut collapsed);
                    let collapsed = alignment(collapsed, start);
                    paths.extend([original, collapsed]);
                }
            }
        }

        // Check all possible combinations
        for ((a, points_a), (b, points_b)) in paths.iter().cartesian_product(&paths) {
            assert_eq!(
                a.intersects(b),
                !points_a.is_disjoint(points_b),
                "{a:?} versus {b:?}",
            );
        }
    }
}
