#[cfg(test)]
use super::Step;
#[cfg(test)]
use std::borrow::Borrow;

use biobit_core_rs::num::PrimUInt;

/// `Op` represents a single operation in a genomic alignment.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Op {
    /// Consumes the first sequence without consuming the second (v).
    GapFirst,
    /// Consumes the second sequence without consuming the first (^).
    GapSecond,
    /// Represents an equivalence, which is ambiguous. It could be a match or mismatch between the target sequences (~).
    /// The interpretation depends on the target problem, e.g. it might represent similar amino acids in two proteins.
    Equivalent,
    /// Represents a match between the sequences (=)
    Match,
    /// Represents a mismatch between the sequences (X)
    Mismatch,
}

impl Op {
    /// Unrolls steps into visited sequence-coordinate pairs, starting at `start`.
    /// Each step includes its starting position and excludes its ending position.
    #[cfg(test)]
    pub(super) fn unroll<Len, Seq1Idx, Seq2Idx>(
        steps: impl IntoIterator<Item: Borrow<Step<Len>>>,
        start: (Seq1Idx, Seq2Idx),
    ) -> impl Iterator<Item = (Seq1Idx, Seq2Idx)>
    where
        Len: PrimUInt,
        Seq1Idx: PrimUInt,
        Seq2Idx: PrimUInt,
    {
        steps
            .into_iter()
            .flat_map(|step| {
                let step = step.borrow();
                let op = *step.op();
                let mut remaining = *step.len();
                std::iter::from_fn(move || {
                    if remaining.is_zero() {
                        return None;
                    }
                    remaining = remaining - Len::one();
                    Some(op)
                })
            })
            .scan(start, |cursor, op| {
                let current = *cursor;
                match op {
                    Op::GapFirst => cursor.0 = cursor.0 + Seq1Idx::one(),
                    Op::GapSecond => cursor.1 = cursor.1 + Seq2Idx::one(),
                    Op::Match | Op::Mismatch | Op::Equivalent => {
                        cursor.0 = cursor.0 + Seq1Idx::one();
                        cursor.1 = cursor.1 + Seq2Idx::one();
                    }
                }
                Some(current)
            })
    }

    /// Returns `true` if the operation is represented by a diagonal movement in the alignment matrix.
    pub fn is_diagonal(&self) -> bool {
        matches!(self, Op::Match | Op::Mismatch | Op::Equivalent)
    }

    /// Returns the symbol representation of the operation.
    pub fn symbol(&self) -> char {
        match self {
            Op::GapFirst => 'v',
            Op::GapSecond => '^',
            Op::Equivalent => '~',
            Op::Match => '=',
            Op::Mismatch => 'X',
        }
    }

    /// Applies the operation to the given sequence indices.
    pub fn apply<Len, Seq1Idx, Seq2Idx>(&self, seq1: &mut Seq1Idx, seq2: &mut Seq2Idx, len: Len)
    where
        Len: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx>,
        Seq1Idx: PrimUInt,
        Seq2Idx: PrimUInt,
    {
        match self {
            Op::GapFirst => *seq1 = *seq1 + len.into(),
            Op::GapSecond => *seq2 = *seq2 + len.into(),
            Op::Equivalent | Op::Mismatch | Op::Match => {
                *seq1 = *seq1 + len.into();
                *seq2 = *seq2 + len.into();
            }
        };
    }
}

impl TryFrom<char> for Op {
    type Error = ();

    /// Tries to convert a character into an `Op`.
    /// Returns an error if the character does not represent a valid operation.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'v' => Ok(Op::GapFirst),
            '^' => Ok(Op::GapSecond),
            '~' => Ok(Op::Equivalent),
            '=' => Ok(Op::Match),
            'X' => Ok(Op::Mismatch),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_symbols() {
        for (op, symbol) in [
            (Op::GapFirst, 'v'),
            (Op::GapSecond, '^'),
            (Op::Equivalent, '~'),
            (Op::Match, '='),
            (Op::Mismatch, 'X'),
        ] {
            assert_eq!(op.symbol(), symbol);
            assert_eq!(Op::try_from(symbol), Ok(op));
        }
        assert_eq!(Op::try_from('a'), Err(()));
    }

    #[test]
    fn test_apply() {
        let mut seq1: u32 = 0;
        let mut seq2: u32 = 0;
        let len: u8 = 1;

        Op::GapFirst.apply(&mut seq1, &mut seq2, len);
        assert_eq!((seq1, seq2), (1, 0));

        Op::GapSecond.apply(&mut seq1, &mut seq2, len);
        assert_eq!((seq1, seq2), (1, 1));

        Op::Equivalent.apply(&mut seq1, &mut seq2, len);
        assert_eq!((seq1, seq2), (2, 2));

        Op::Match.apply(&mut seq1, &mut seq2, len);
        assert_eq!((seq1, seq2), (3, 3));

        Op::Mismatch.apply(&mut seq1, &mut seq2, len);
        assert_eq!((seq1, seq2), (4, 4));
    }
}
