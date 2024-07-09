use std::iter::Peekable;

use crate::num::PrimUInt;

use super::offset::Offset;
use super::op::Op;

/// An alignment step in the genomic alignment
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Step<Len: PrimUInt = u8> {
    /// The alignment operation
    pub op: Op,
    /// The length of the operation, e.g. the number of consequent matches or gaps
    pub len: Len,
}

impl<Len: PrimUInt> Step<Len> {
    /// Optimize the sequence of steps by merging identical operations to minimize the memory usage.
    /// If the sum of the lengths exceeds the maximum value of the step size, the step is split.
    pub fn optimize(steps: &mut Vec<Step<Len>>) {
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

    /// Repack steps by using a different step size and optimizing the overall memory usage.
    /// Identical operations are merged as much as possible while steps exceeding the maximum length are split.
    pub fn repack<T: PrimUInt>() -> Step<T> {
        todo!("Implement repack")
    }
}

/// An alignment step with the start position in sequence coordinates
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct AlignedStep<Len: PrimUInt = u8, Seq1Offset: PrimUInt = u64, Seq2Offset: PrimUInt = u64> {
    pub step: Step<Len>,
    pub start: Offset<Seq1Offset, Seq2Offset>,
}

impl<Len: PrimUInt, Seq1Offset: PrimUInt, Seq2Offset: PrimUInt>
    AlignedStep<Len, Seq1Offset, Seq2Offset>
where
    Len: Into<Seq1Offset> + Into<Seq2Offset>,
{
    /// Get the end position of the step in sequence coordinates (e.g. the alignment position after applying the step)
    pub fn end(&self) -> Offset<Seq1Offset, Seq2Offset> {
        let mut end = self.start;
        match self.step.op {
            Op::GapFirst => end.seq1 = end.seq1 + self.step.len.into(),
            Op::GapSecond => end.seq2 = end.seq2 + self.step.len.into(),
            Op::Equivalent | Op::Mismatch | Op::Match => {
                end.seq1 = end.seq1 + self.step.len.into();
                end.seq2 = end.seq2 + self.step.len.into();
            }
        };
        end
    }
}

/// An iterator that keeps track of the current offset in the alignment
pub struct AlignedStepIterator<
    'a,
    T: Iterator<Item = &'a Step<Len>>,
    Len: PrimUInt + 'a = u8,
    Seq1Offset: PrimUInt = u64,
    Seq2Offset: PrimUInt = u64,
> {
    pub iter: Peekable<T>,
    pub offset: Offset<Seq1Offset, Seq2Offset>,
}

impl<
        'a,
        T: Iterator<Item = &'a Step<Len>>,
        Len: PrimUInt,
        Seq1Offset: PrimUInt,
        Seq2Offset: PrimUInt,
    > Iterator for AlignedStepIterator<'a, T, Len, Seq1Offset, Seq2Offset>
where
    Len: Into<Seq1Offset> + Into<Seq2Offset>,
{
    type Item = AlignedStep<Len, Seq1Offset, Seq2Offset>;

    fn next(&mut self) -> Option<Self::Item> {
        let step = match self.iter.next() {
            None => {
                return None;
            }
            Some(x) => AlignedStep {
                start: self.offset,
                step: *x,
            },
        };

        self.offset.seq1 = self.offset.seq1 + step.step.len.into();
        self.offset.seq2 = self.offset.seq2 + step.step.len.into();
        return Some(step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_step_end() {
        let step = AlignedStep::<u8> {
            step: Step {
                op: Op::Match,
                len: 5,
            },
            start: Offset { seq1: 0, seq2: 0 },
        };
        assert_eq!(step.end(), Offset { seq1: 5, seq2: 5 });
    }

    #[test]
    fn test_aligned_step_iterator() {
        let steps: Vec<Step<u8>> = vec![
            Step {
                op: Op::Match,
                len: 1,
            },
            Step {
                op: Op::GapFirst,
                len: 2,
            },
            Step {
                op: Op::Match,
                len: 3,
            },
        ];
        let mut iter = AlignedStepIterator::<_, _, u64, u64> {
            iter: steps.iter().peekable(),
            offset: Offset { seq1: 10, seq2: 0 },
        };
        assert_eq!(
            iter.next(),
            Some(AlignedStep {
                step: Step {
                    op: Op::Match,
                    len: 1
                },
                start: Offset { seq1: 10, seq2: 0 },
            })
        );
        assert_eq!(
            iter.next(),
            Some(AlignedStep {
                step: Step {
                    op: Op::GapFirst,
                    len: 2
                },
                start: Offset { seq1: 11, seq2: 1 },
            })
        );
        assert_eq!(
            iter.next(),
            Some(AlignedStep {
                step: Step {
                    op: Op::Match,
                    len: 3
                },
                start: Offset { seq1: 13, seq2: 3 },
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_optimize() {
        let mut steps: Vec<Step<u8>> = vec![
            Step {
                op: Op::Match,
                len: 10,
            },
            Step {
                op: Op::Match,
                len: 20,
            },
            Step {
                op: Op::Match,
                len: 30,
            },
            Step {
                op: Op::Match,
                len: 40,
            },
            Step {
                op: Op::Match,
                len: 50,
            },
            Step {
                op: Op::GapFirst,
                len: 200,
            },
            Step {
                op: Op::GapFirst,
                len: 100,
            },
            Step {
                op: Op::Match,
                len: 15,
            },
            Step {
                op: Op::Match,
                len: 15,
            },
            Step {
                op: Op::Match,
                len: 15,
            },
        ];
        Step::optimize(&mut steps);
        assert_eq!(
            steps,
            vec![
                Step {
                    op: Op::Match,
                    len: 150
                },
                Step {
                    op: Op::GapFirst,
                    len: 255
                },
                Step {
                    op: Op::GapFirst,
                    len: 45
                },
                Step {
                    op: Op::Match,
                    len: 45
                },
            ]
        );
    }
}
