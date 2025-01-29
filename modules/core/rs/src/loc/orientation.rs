use std::fmt::Display;

use super::strand::Strand;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

/// A type representing the orientation of an object in the genome
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(i8)]
pub enum Orientation {
    /// Object is located on the forward strand, also known as the positive strand or Watson strand.
    Forward = 1,
    /// Object is located on the reverse strand, also known as the negative strand or Crick strand.
    Reverse = -1,
    /// Object is located on both strands (e.g. a CpG island or a bidirectional promoter).
    #[default]
    Dual = 0,
}

impl Orientation {
    /// Flip the orientation from forward to reverse or vice versa. Dual orientation remains the same.
    pub fn flip(&mut self) -> &mut Self {
        *self = match self {
            Orientation::Forward => Orientation::Reverse,
            Orientation::Reverse => Orientation::Forward,
            Orientation::Dual => Orientation::Dual,
        };
        self
    }

    /// New orientation that is the opposite of the current one. Dual orientation remains the same.
    pub fn flipped(&self) -> Self {
        match self {
            Orientation::Forward => Orientation::Reverse,
            Orientation::Reverse => Orientation::Forward,
            Orientation::Dual => Orientation::Dual,
        }
    }

    /// Get the symbolic representation of the strand.
    pub fn symbol(&self) -> char {
        match self {
            Orientation::Forward => '+',
            Orientation::Reverse => '-',
            Orientation::Dual => '=',
        }
    }
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl TryFrom<char> for Orientation {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Orientation::Forward),
            '-' => Ok(Orientation::Reverse),
            '=' => Ok(Orientation::Dual),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Orientation {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Orientation::Forward),
            "-" => Ok(Orientation::Reverse),
            "=" => Ok(Orientation::Dual),
            _ => Err(()),
        }
    }
}

impl TryFrom<i8> for Orientation {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Orientation::Forward),
            -1 => Ok(Orientation::Reverse),
            0 => Ok(Orientation::Dual),
            _ => Err(()),
        }
    }
}

impl From<Strand> for Orientation {
    fn from(value: Strand) -> Self {
        match value {
            Strand::Forward => Orientation::Forward,
            Strand::Reverse => Orientation::Reverse,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_flip() {
        assert_eq!(*Orientation::Forward.flip(), Orientation::Reverse);
        assert_eq!(*Orientation::Reverse.flip(), Orientation::Forward);
        assert_eq!(*Orientation::Dual.flip(), Orientation::Dual);
    }

    #[test]
    fn test_orientation_flipped() {
        assert_eq!(Orientation::Forward.flipped(), Orientation::Reverse);
        assert_eq!(Orientation::Reverse.flipped(), Orientation::Forward);
        assert_eq!(Orientation::Dual.flipped(), Orientation::Dual);
    }

    #[test]
    fn test_orientation_symbol() {
        assert_eq!(Orientation::Forward.symbol(), '+');
        assert_eq!(Orientation::Reverse.symbol(), '-');
        assert_eq!(Orientation::Dual.symbol(), '=');
    }

    #[test]
    fn test_orientation_display() {
        assert_eq!(format!("{}", Orientation::Forward), "+");
        assert_eq!(format!("{}", Orientation::Reverse), "-");
        assert_eq!(format!("{}", Orientation::Dual), "=");
    }

    #[test]
    fn test_orientation_try_from() {
        assert_eq!(Orientation::try_from('+'), Ok(Orientation::Forward));
        assert_eq!(Orientation::try_from('-'), Ok(Orientation::Reverse));
        assert_eq!(Orientation::try_from('='), Ok(Orientation::Dual));
        assert_eq!(Orientation::try_from('x'), Err(()));

        assert_eq!(Orientation::try_from(1), Ok(Orientation::Forward));
        assert_eq!(Orientation::try_from(-1), Ok(Orientation::Reverse));
        assert_eq!(Orientation::try_from(0), Ok(Orientation::Dual));
        assert_eq!(Orientation::try_from(2), Err(()));
    }

    #[test]
    fn test_orientation_try_from_strand() {
        assert_eq!(
            Orientation::try_from(Strand::Forward),
            Ok(Orientation::Forward)
        );
        assert_eq!(
            Orientation::try_from(Strand::Reverse),
            Ok(Orientation::Reverse)
        );
    }
}
