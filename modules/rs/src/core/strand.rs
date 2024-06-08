use std::fmt::Display;

use super::orientation::Orientation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strand {
    /// The forward strand, also known as the positive strand or Watson strand.
    Forward,
    /// The reverse strand, also known as the negative strand or Crick strand.
    Reverse,
}

impl Strand {
    /// Flip the strand from forward to reverse or vice versa.
    pub fn flip(&self) -> Strand {
        match self {
            Strand::Forward => Strand::Reverse,
            Strand::Reverse => Strand::Forward,
        }
    }

    /// Get the symbolic representation of the strand.
    pub fn symbol(&self) -> char {
        match self {
            Strand::Forward => '+',
            Strand::Reverse => '-',
        }
    }
}

impl Display for Strand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl TryFrom<char> for Strand {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Strand::Forward),
            '-' => Ok(Strand::Reverse),
            _ => Err(()),
        }
    }
}

impl TryFrom<Orientation> for Strand {
    type Error = ();

    fn try_from(value: Orientation) -> Result<Self, Self::Error> {
        match value {
            Orientation::Forward => Ok(Strand::Forward),
            Orientation::Reverse => Ok(Strand::Reverse),
            _ => Err(()),
        }
    }
}


/// A trait for types that can be uniquely associated with a strand.
pub trait HasStrand {
    /// Get the strand of the object.
    fn strand(&self) -> Strand;
}

mod tests {
    use super::*;

    #[test]
    fn test_strand_flip() {
        assert_eq!(Strand::Forward.flip(), Strand::Reverse);
        assert_eq!(Strand::Reverse.flip(), Strand::Forward);
    }

    #[test]
    fn test_strand_symbol() {
        assert_eq!(Strand::Forward.symbol(), '+');
        assert_eq!(Strand::Reverse.symbol(), '-');
    }

    #[test]
    fn test_strand_display() {
        assert_eq!(format!("{}", Strand::Forward), "+");
        assert_eq!(format!("{}", Strand::Reverse), "-");
    }

    #[test]
    fn test_strand_try_from() {
        assert_eq!(Strand::try_from('+'), Ok(Strand::Forward));
        assert_eq!(Strand::try_from('-'), Ok(Strand::Reverse));
        assert_eq!(Strand::try_from('x'), Err(()));
    }

    #[test]
    fn test_strand_try_from_orientation() {
        assert_eq!(Strand::try_from(Orientation::Forward), Ok(Strand::Forward));
        assert_eq!(Strand::try_from(Orientation::Reverse), Ok(Strand::Reverse));
        assert_eq!(Strand::try_from(Orientation::Dual), Err(()));
    }
}