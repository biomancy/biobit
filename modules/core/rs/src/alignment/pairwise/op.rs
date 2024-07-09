/// `Op` represents a single operation in a genomic alignment.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Op {
    /// Represents a gap in the first sequence (v)
    GapFirst,
    /// Represents a gap in the second sequence (^)
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

    /// Tests the conversion from a character to an `Op`.
    #[test]
    fn test_try_from_char() {
        assert_eq!(Op::try_from('v'), Ok(Op::GapFirst));
        assert_eq!(Op::try_from('^'), Ok(Op::GapSecond));
        assert_eq!(Op::try_from('~'), Ok(Op::Equivalent));
        assert_eq!(Op::try_from('='), Ok(Op::Match));
        assert_eq!(Op::try_from('X'), Ok(Op::Mismatch));
        assert_eq!(Op::try_from('a'), Err(()));
    }
}
