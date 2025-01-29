use std::borrow::Borrow;
use std::fmt::Display;
use std::marker::PhantomData;

use derive_getters::{Dissolve, Getters};
use derive_more::{Constructor, From, Into};
use eyre::Result;

use biobit_core_rs::num::PrimUInt;

use super::offset::Offset;
use super::op::Op;

/// An alignment step in the genomic alignment
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Getters, Dissolve)]
pub struct Step<Len: PrimUInt> {
    /// The length of the operation, e.g. the number of consequent matches or gaps.
    /// Guaranteed to be greater than zero.
    len: Len,
    /// The alignment operation
    op: Op,
}

impl<Len: PrimUInt> Step<Len> {
    pub fn new(op: Op, len: Len) -> Result<Self> {
        if len.is_zero() {
            return Err(eyre::eyre!("Step length must be greater than zero"));
        }
        Ok(Self { len, op })
    }

    /// Optimize the sequence of steps by collapsing identical operations to minimize the memory usage.
    /// If the sum of the lengths exceeds the maximum value of the step size, the step is divided accordingly.
    pub fn collapse(steps: &mut Vec<Step<Len>>) {
        if steps.is_empty() || steps.len() == 1 {
            return;
        }

        let (mut writep, mut readp) = (0, 1);

        while readp < steps.len() {
            if steps[writep].op == steps[readp].op {
                // If the sum of the lengths exceeds the maximum value of the step size, split the step
                match steps[writep].len.checked_add(&steps[readp].len) {
                    Some(x) => steps[writep].len = x,
                    None => {
                        steps[readp].len =
                            steps[readp].len - (Len::max_value() - steps[writep].len);
                        debug_assert!(steps[readp].len > Len::zero());
                        steps[writep].len = Len::max_value();

                        writep += 1;
                        steps[writep] = steps[readp];
                    }
                }
            } else {
                writep += 1;
                steps[writep] = steps[readp];
            }
            readp += 1;
        }
        steps.truncate(writep + 1);
    }

    pub fn rle_string(steps: impl Iterator<Item: Borrow<Step<Len>>>) -> String
    where
        Len: Display,
    {
        // 2 symbols is an average length of a step
        // 1 is the length of the symbol
        let hint = match steps.size_hint() {
            (_, Some(upper)) => upper * 3,
            (lower, _) => lower * 3,
        };

        let mut result = String::with_capacity(hint);
        for step in steps {
            let step = step.borrow();
            result.push_str(&step.len().to_string());
            result.push(step.op().symbol());
        }
        result
    }
}

/// A tracked alignment step with known start position (offset) in the sequence coordinates
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Getters, Dissolve, Constructor, From, Into)]
pub struct StepWithOffset<
    Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
> {
    pub step: Step<Len>,
    pub start: Offset<Seq1Idx, Seq2Idx>,
}

impl<Len, Seq1Idx, Seq2Idx> StepWithOffset<Len, Seq1Idx, Seq2Idx>
where
    Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
{
    pub fn intersects<Accumulator: PrimUInt>(&self, other: &Self) -> bool
    where
        Len: Into<Accumulator>,
        Seq1Idx: Into<Accumulator>,
        Seq2Idx: Into<Accumulator>,
    {
        // Can this mess turned into something more readable / faster?
        match (self.step.op, other.step.op) {
            (Op::GapFirst, Op::GapFirst) => {
                let self_end_seq1 = self.start.seq1 + self.step.len.into();
                let other_end_seq1 = other.start.seq1 + other.step.len.into();

                self.start.seq2 == other.start.seq2
                    && self.start.seq1 < other_end_seq1
                    && other.start.seq1 < self_end_seq1
            }
            (Op::GapSecond, Op::GapSecond) => {
                let other_end_seq2 = other.start.seq2 + other.step.len.into();
                let self_end_seq2 = self.start.seq2 + self.step.len.into();

                self.start.seq1 == other.start.seq1
                    && self.start.seq2 < other_end_seq2
                    && other.start.seq2 < self_end_seq2
            }
            (Op::GapFirst, Op::GapSecond) => {
                let other_end_seq2 = other.start.seq2 + other.step.len.into();
                let self_end_seq1 = self.start.seq1 + self.step.len.into();

                self.start.seq2 >= other.start.seq2
                    && self.start.seq2 < other_end_seq2
                    && other.start.seq1 >= self.start.seq1
                    && other.start.seq1 < self_end_seq1
            }
            (Op::GapSecond, Op::GapFirst) => {
                let other_end_seq1 = other.start.seq1 + other.step.len.into();
                let self_end_seq2 = self.start.seq2 + self.step.len.into();

                self.start.seq1 >= other.start.seq1
                    && self.start.seq1 < other_end_seq1
                    && other.start.seq2 >= self.start.seq2
                    && other.start.seq2 < self_end_seq2
            }
            // Within the same range & matched diagonal point is behind the gap end
            (Op::Match | Op::Mismatch | Op::Equivalent, Op::GapFirst) => {
                let self_end_seq2 = self.start.seq2 + self.step.len.into();
                if other.start.seq2 < self.start.seq2 || other.start.seq2 >= self_end_seq2 {
                    return false;
                }

                let other_end_seq1: Accumulator = other.start.seq1.into() + other.step.len.into();
                let self_seq1_shifted: Accumulator =
                    self.start.seq1.into() + (other.start.seq2 - self.start.seq2).into();

                self_seq1_shifted < other_end_seq1 && self_seq1_shifted >= other.start.seq1.into()
            }
            (Op::GapFirst, Op::Match | Op::Mismatch | Op::Equivalent) => {
                let other_end_seq2 = other.start.seq2 + other.step.len.into();
                if self.start.seq2 < other.start.seq2 || self.start.seq2 >= other_end_seq2 {
                    return false;
                }

                let self_end_seq1: Accumulator = self.start.seq1.into() + self.step.len.into();
                let other_seq1_shifted: Accumulator =
                    other.start.seq1.into() + (self.start.seq2 - other.start.seq2).into();

                other_seq1_shifted < self_end_seq1 && other_seq1_shifted >= self.start.seq1.into()
            }
            (Op::Match | Op::Mismatch | Op::Equivalent, Op::GapSecond) => {
                let self_end_seq1 = self.start.seq1 + self.step.len.into();
                if other.start.seq1 < self.start.seq1 || other.start.seq1 >= self_end_seq1 {
                    return false;
                }

                let other_end_seq2: Accumulator = other.start.seq2.into() + other.step.len.into();
                let self_seq2_shifted: Accumulator =
                    self.start.seq2.into() + (other.start.seq1 - self.start.seq1).into();

                self_seq2_shifted < other_end_seq2 && self_seq2_shifted >= other.start.seq2.into()
            }
            (Op::GapSecond, Op::Match | Op::Mismatch | Op::Equivalent) => {
                let other_end_seq1 = other.start.seq1 + other.step.len.into();
                if self.start.seq1 < other.start.seq1 || self.start.seq1 >= other_end_seq1 {
                    return false;
                }

                let self_end_seq2: Accumulator = self.start.seq2.into() + self.step.len.into();
                let other_seq2_shifted: Accumulator =
                    other.start.seq2.into() + (self.start.seq1 - other.start.seq1).into();

                other_seq2_shifted < self_end_seq2 && other_seq2_shifted >= self.start.seq2.into()
            }
            // Same diagonal + overlap
            (
                Op::Match | Op::Mismatch | Op::Equivalent,
                Op::Match | Op::Mismatch | Op::Equivalent,
            ) => {
                let self_end_seq1 = self.start.seq1 + self.step.len.into();
                let other_end_seq1 = other.start.seq1 + other.step.len.into();

                // Projection for one of the axis must overlap (any axis)
                if self.start.seq1 >= other_end_seq1 || self_end_seq1 <= other.start.seq1 {
                    return false;
                }

                // Diff between segments must be identical
                let seq1_diff = if self.start.seq1 > other.start.seq1 {
                    self.start.seq1 - other.start.seq1
                } else {
                    other.start.seq1 - self.start.seq1
                };

                let seq2_diff = if self.start.seq2 > other.start.seq2 {
                    self.start.seq2 - other.start.seq2
                } else {
                    other.start.seq2 - self.start.seq2
                };

                seq1_diff.into() == seq2_diff.into()
            }
        }
    }

    /// Get the end position of the step in sequence coordinates (e.g. the alignment position after applying the step)
    pub fn end(&self) -> Offset<Seq1Idx, Seq2Idx> {
        let mut end = self.start;
        self.step
            .op
            .apply(&mut end.seq1, &mut end.seq2, self.step.len);
        end
    }
}

/// An iterator that keeps track of the current offset in the alignment
pub struct StepsWithOffsetsIterator<
    T: Iterator<Item: Into<Step<Len>>>,
    Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
> {
    iter: T,
    offset: Offset<Seq1Idx, Seq2Idx>,
    phantom_data: PhantomData<Len>,
}

impl<T, Len, Seq1Idx, Seq2Idx> StepsWithOffsetsIterator<T, Len, Seq1Idx, Seq2Idx>
where
    T: Iterator<Item: Into<Step<Len>>>,
    Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
{
    pub fn new(iter: T, offset: Offset<Seq1Idx, Seq2Idx>) -> Self {
        Self {
            iter,
            offset,
            phantom_data: Default::default(),
        }
    }
}

impl<T, Len, Seq1Idx, Seq2Idx> Iterator for StepsWithOffsetsIterator<T, Len, Seq1Idx, Seq2Idx>
where
    T: Iterator<Item: Into<Step<Len>>>,
    Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
    Seq1Idx: PrimUInt,
    Seq2Idx: PrimUInt,
{
    type Item = StepWithOffset<Len, Seq1Idx, Seq2Idx>;

    fn next(&mut self) -> Option<Self::Item> {
        let step = match self.iter.next() {
            None => return None,
            Some(x) => StepWithOffset {
                start: self.offset,
                step: x.into(),
            },
        };

        step.step
            .op
            .apply(&mut self.offset.seq1, &mut self.offset.seq2, step.step.len);
        Some(step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_steps<L: PrimUInt>(steps: &[(Op, L)]) -> Vec<Step<L>> {
        steps
            .iter()
            .map(|(op, len)| Step::new(*op, *len).unwrap())
            .collect()
    }

    #[test]
    fn test_aligned_step_end() {
        let step = StepWithOffset::<u8, usize, usize> {
            step: Step::new(Op::Match, 5).unwrap(),
            start: Offset { seq1: 0, seq2: 0 },
        };
        assert_eq!(step.end(), Offset { seq1: 5, seq2: 5 });
    }

    #[test]
    fn test_aligned_step_iterator() -> Result<()> {
        let steps = to_steps::<u8>(&[(Op::Match, 1), (Op::GapFirst, 2), (Op::Match, 3)]);

        let mut iter = StepsWithOffsetsIterator::<_, _, u64, u64> {
            iter: steps.iter().cloned(),
            offset: Offset { seq1: 10, seq2: 0 },
            phantom_data: Default::default(),
        };
        assert_eq!(
            iter.next(),
            Some(StepWithOffset {
                step: Step::new(Op::Match, 1)?,
                start: Offset { seq1: 10, seq2: 0 },
            })
        );
        assert_eq!(
            iter.next(),
            Some(StepWithOffset {
                step: Step::new(Op::GapFirst, 2)?,
                start: (11, 1).into(),
            })
        );
        assert_eq!(
            iter.next(),
            Some(StepWithOffset {
                step: Step::new(Op::Match, 3)?,
                start: Offset { seq1: 13, seq2: 1 },
            })
        );
        assert_eq!(iter.next(), None);

        Ok(())
    }

    #[test]
    fn test_step_optimize() {
        let mut steps = to_steps::<u8>(&[
            (Op::Match, 10),
            (Op::Match, 20),
            (Op::Match, 30),
            (Op::Match, 40),
            (Op::Match, 50),
            (Op::GapFirst, 200),
            (Op::GapFirst, 100),
            (Op::Match, 15),
            (Op::Match, 15),
            (Op::Match, 15),
        ]);

        Step::collapse(&mut steps);
        let expected = to_steps::<u8>(&[
            (Op::Match, 150),
            (Op::GapFirst, 255),
            (Op::GapFirst, 45),
            (Op::Match, 45),
        ]);

        assert_eq!(steps, expected);
    }

    #[test]
    fn test_rle_string() {
        // Empty input -> empty output
        assert_eq!(Step::rle_string(std::iter::empty::<&Step<u8>>()), "");

        // Single step
        let steps = to_steps::<u8>(&[(Op::Match, 1)]);
        assert_eq!(Step::rle_string(steps.iter()), "1=");

        // Multiple steps
        let steps = to_steps::<u8>(&[(Op::Match, 1), (Op::GapFirst, 2), (Op::Match, 3)]);
        assert_eq!(Step::rle_string(steps.iter()), "1=2v3=");
    }

    fn to_step_with_offset(op: Op, len: u8, seq1: u32, seq2: u32) -> StepWithOffset<u8, u32, u32> {
        StepWithOffset {
            step: Step::new(op, len).unwrap(),
            start: Offset { seq1, seq2 },
        }
    }

    #[test]
    fn test_intersection_with_gap_first() {
        let step = to_step_with_offset(Op::GapFirst, 5, 10, 10);

        // Negative cases
        for bait in &[
            // GapFirst
            to_step_with_offset(Op::GapFirst, 5, 5, 10), // Left
            to_step_with_offset(Op::GapFirst, 5, 15, 10), // Right
            to_step_with_offset(Op::GapFirst, 5, 10, 11), // Top
            to_step_with_offset(Op::GapFirst, 5, 10, 9), // Bottom
            // GapSecond
            to_step_with_offset(Op::GapSecond, 5, 9, 10), // Left
            to_step_with_offset(Op::GapSecond, 5, 15, 10), // Right
            to_step_with_offset(Op::GapSecond, 5, 10, 11), // Top
            to_step_with_offset(Op::GapSecond, 5, 10, 5), // Bottom
        ] {
            assert!(!step.intersects::<u64>(&bait));
            assert!(!bait.intersects::<u64>(&step));
        }

        for op in [Op::Match, Op::Equivalent, Op::Mismatch] {
            for bait in &[
                to_step_with_offset(op, 5, 8, 9),   // Left
                to_step_with_offset(op, 5, 14, 9),  // Right
                to_step_with_offset(op, 5, 12, 12), // Top
                to_step_with_offset(op, 5, 4, 4),   // Bottom
                to_step_with_offset(op, 6, 4, 4),   // Bottom
            ] {
                assert!(!step.intersects::<u64>(&bait));
                assert!(!bait.intersects::<u64>(&step));
            }
        }

        // Positive cases
        for bait in &[
            // GapFirst
            to_step_with_offset(Op::GapFirst, 5, 6, 10), // Left
            to_step_with_offset(Op::GapFirst, 5, 8, 10), // Center
            to_step_with_offset(Op::GapFirst, 5, 14, 10), // Right
            // GapSecond
            to_step_with_offset(Op::GapSecond, 5, 10, 7), // Left
            to_step_with_offset(Op::GapSecond, 5, 12, 7), // Center
            to_step_with_offset(Op::GapSecond, 5, 14, 7), // Right
        ] {
            assert!(step.intersects::<u64>(&bait));
            assert!(bait.intersects::<u64>(&step));
        }

        for op in [Op::Match, Op::Mismatch, Op::Equivalent] {
            for bait in &[
                to_step_with_offset(op, 10, 5, 5),   // Left
                to_step_with_offset(op, 10, 10, 10), // Left
                to_step_with_offset(op, 10, 9, 9),   // Left
                to_step_with_offset(op, 10, 9, 5),   // Right
                to_step_with_offset(op, 10, 14, 10), // Right
                to_step_with_offset(op, 10, 13, 9),  // Right
                to_step_with_offset(op, 10, 8, 6),   // Center
                to_step_with_offset(op, 5, 8, 6),    // Center
            ] {
                assert!(step.intersects::<u64>(&bait));
                assert!(bait.intersects::<u64>(&step));
            }
        }
    }

    #[test]
    fn test_intersection_with_gap_second() {
        let step = to_step_with_offset(Op::GapSecond, 5, 10, 10);

        // Negative cases
        for bait in &[
            // GapFirst
            to_step_with_offset(Op::GapFirst, 5, 5, 9), // Left
            to_step_with_offset(Op::GapFirst, 5, 5, 10), // Left
            to_step_with_offset(Op::GapFirst, 5, 5, 14), // Left
            to_step_with_offset(Op::GapFirst, 5, 11, 9), // Right
            to_step_with_offset(Op::GapFirst, 5, 11, 10), // Right
            to_step_with_offset(Op::GapFirst, 5, 11, 14), // Right
            to_step_with_offset(Op::GapFirst, 5, 10, 15), // Top
            to_step_with_offset(Op::GapFirst, 5, 10, 16), // Top
            to_step_with_offset(Op::GapFirst, 5, 10, 9), // Bottom
            to_step_with_offset(Op::GapFirst, 5, 10, 8), // Bottom
            // GapSecond
            to_step_with_offset(Op::GapSecond, 5, 9, 9), // Left
            to_step_with_offset(Op::GapSecond, 5, 9, 10), // Left
            to_step_with_offset(Op::GapSecond, 5, 11, 9), // Right
            to_step_with_offset(Op::GapSecond, 5, 11, 10), // Right
            to_step_with_offset(Op::GapSecond, 5, 10, 15), // Top
            to_step_with_offset(Op::GapSecond, 5, 10, 5), // Bottom
        ] {
            assert!(!step.intersects::<u64>(&bait));
            assert!(!bait.intersects::<u64>(&step));
        }

        for op in [Op::Match, Op::Equivalent, Op::Mismatch] {
            for bait in &[
                to_step_with_offset(op, 5, 4, 4),   // Left
                to_step_with_offset(op, 5, 5, 5),   // Left
                to_step_with_offset(op, 5, 5, 8),   // Left
                to_step_with_offset(op, 5, 5, 10),  // Left
                to_step_with_offset(op, 5, 10, 16), // Right
                to_step_with_offset(op, 5, 10, 15), // Right
                to_step_with_offset(op, 5, 11, 13), // Right
                to_step_with_offset(op, 5, 8, 7),   // Right
            ] {
                assert!(!step.intersects::<u64>(&bait));
                assert!(!bait.intersects::<u64>(&step));
            }
        }

        // Positive cases
        for bait in &[
            // GapFirst
            to_step_with_offset(Op::GapFirst, 5, 10, 10), // Left
            to_step_with_offset(Op::GapFirst, 5, 8, 12),  // Center
            to_step_with_offset(Op::GapFirst, 5, 10, 12), // Center
            to_step_with_offset(Op::GapFirst, 5, 8, 14),  // Top
            // GapSecond
            to_step_with_offset(Op::GapSecond, 5, 10, 6), // Bottom
            to_step_with_offset(Op::GapSecond, 5, 10, 10), // Identical
            to_step_with_offset(Op::GapSecond, 5, 10, 14), // Top
        ] {
            assert!(step.intersects::<u64>(&bait));
            assert!(bait.intersects::<u64>(&step));
        }

        for op in [Op::Match, Op::Mismatch, Op::Equivalent] {
            for bait in &[
                // Bottom
                to_step_with_offset(op, 10, 5, 5),
                to_step_with_offset(op, 10, 10, 10),
                // Left
                to_step_with_offset(op, 10, 1, 2),
                to_step_with_offset(op, 10, 1, 4),
                // Top
                to_step_with_offset(op, 10, 10, 14),
                to_step_with_offset(op, 11, 0, 4),
                // Right
                to_step_with_offset(op, 10, 10, 13),
                // Center
                to_step_with_offset(op, 10, 6, 8),
                to_step_with_offset(op, 15, 0, 1),
            ] {
                assert!(step.intersects::<u64>(&bait));
                assert!(bait.intersects::<u64>(&step));
            }
        }
    }

    #[test]
    fn test_intersection_with_diagonal() {
        for op in [Op::Match, Op::Mismatch, Op::Equivalent] {
            let step = to_step_with_offset(op, 10, 10, 10);

            // Negative cases
            for bait in &[
                // GapFirst
                // Bottom
                to_step_with_offset(Op::GapFirst, 10, 5, 5),
                to_step_with_offset(Op::GapFirst, 10, 5, 9),
                // Top
                to_step_with_offset(Op::GapFirst, 10, 15, 20),
                to_step_with_offset(Op::GapFirst, 10, 15, 25),
                // Left
                to_step_with_offset(Op::GapFirst, 10, 0, 10),
                to_step_with_offset(Op::GapFirst, 10, 5, 15),
                to_step_with_offset(Op::GapFirst, 10, 5, 16),
                to_step_with_offset(Op::GapFirst, 10, 10, 20),
                // Right
                to_step_with_offset(Op::GapFirst, 10, 11, 10),
                to_step_with_offset(Op::GapFirst, 10, 16, 15),
                to_step_with_offset(Op::GapFirst, 10, 30, 30),
                to_step_with_offset(Op::GapFirst, 10, 20, 20),
                // GapSecond
                // Bottom
                to_step_with_offset(Op::GapSecond, 10, 5, 5),
                to_step_with_offset(Op::GapSecond, 10, 9, 5),
                // Top
                to_step_with_offset(Op::GapSecond, 10, 20, 15),
                to_step_with_offset(Op::GapSecond, 10, 25, 15),
                // Left
                to_step_with_offset(Op::GapSecond, 10, 10, 0),
                to_step_with_offset(Op::GapSecond, 10, 15, 5),
                to_step_with_offset(Op::GapSecond, 10, 16, 5),
                to_step_with_offset(Op::GapSecond, 10, 20, 10),
                // Right
                to_step_with_offset(Op::GapSecond, 10, 10, 11),
                to_step_with_offset(Op::GapSecond, 10, 15, 16),
                to_step_with_offset(Op::GapSecond, 10, 30, 30),
                to_step_with_offset(Op::GapSecond, 10, 20, 20),
            ] {
                assert!(!step.intersects::<u64>(&bait));
                assert!(!bait.intersects::<u64>(&step));
            }

            for op in [Op::Match, Op::Equivalent, Op::Mismatch] {
                for bait in &[
                    // Parallel
                    to_step_with_offset(op, 10, 9, 10),
                    to_step_with_offset(op, 10, 10, 9),
                    // Before/After
                    to_step_with_offset(op, 10, 20, 20),
                    to_step_with_offset(op, 10, 0, 0),
                    // Diagonal
                    to_step_with_offset(op, 10, 15, 13),
                    to_step_with_offset(op, 10, 9, 0),
                ] {
                    assert!(!step.intersects::<u64>(&bait));
                    assert!(!bait.intersects::<u64>(&step));
                }
            }

            // Positive cases
            for bait in &[
                // GapFirst
                // Touching
                to_step_with_offset(Op::GapFirst, 11, 0, 10),
                to_step_with_offset(Op::GapFirst, 11, 5, 15),
                to_step_with_offset(Op::GapFirst, 11, 9, 19),
                // Crossing
                to_step_with_offset(Op::GapFirst, 15, 0, 10),
                to_step_with_offset(Op::GapFirst, 15, 5, 15),
                to_step_with_offset(Op::GapFirst, 15, 9, 19),
                // Starting
                to_step_with_offset(Op::GapFirst, 10, 10, 10),
                to_step_with_offset(Op::GapFirst, 10, 15, 15),
                to_step_with_offset(Op::GapFirst, 10, 19, 19),
                // GapSecond
                // Touching
                to_step_with_offset(Op::GapSecond, 11, 10, 0),
                to_step_with_offset(Op::GapSecond, 11, 15, 5),
                to_step_with_offset(Op::GapSecond, 11, 19, 9),
                // Crossing
                to_step_with_offset(Op::GapSecond, 15, 10, 0),
                to_step_with_offset(Op::GapSecond, 15, 15, 5),
                to_step_with_offset(Op::GapSecond, 15, 19, 9),
                // Starting
                to_step_with_offset(Op::GapSecond, 10, 10, 10),
                to_step_with_offset(Op::GapSecond, 10, 15, 15),
                to_step_with_offset(Op::GapSecond, 10, 19, 19),
            ] {
                assert!(step.intersects::<u64>(&bait));
                assert!(bait.intersects::<u64>(&step));
            }

            for op in [Op::Match, Op::Equivalent, Op::Mismatch] {
                for bait in &[
                    to_step_with_offset(op, 6, 5, 5),
                    to_step_with_offset(op, 10, 19, 19),
                    to_step_with_offset(op, 10, 10, 10),
                    to_step_with_offset(op, 2, 14, 14),
                ] {
                    assert!(step.intersects::<u64>(&bait));
                    assert!(bait.intersects::<u64>(&step));
                }
            }
        }
    }
}
