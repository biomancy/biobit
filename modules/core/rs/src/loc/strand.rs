use std::fmt::Display;

use super::orientation::Orientation;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(i8)]
pub enum Strand {
    /// The forward strand, also known as the positive strand or Watson strand.
    Forward = 1,
    /// The reverse strand, also known as the negative strand or Crick strand.
    Reverse = -1,
}

impl Strand {
    /// Flip the strand from forward to reverse or vice versa.
    pub fn flip(&mut self) -> &mut Self {
        *self = match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        };
        self
    }

    /// New strand that is the opposite of the current one.
    pub fn flipped(&self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }

    /// Get the symbolic representation of the strand.
    pub fn symbol(&self) -> char {
        match self {
            Self::Forward => '+',
            Self::Reverse => '-',
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
            '+' => Ok(Self::Forward),
            '-' => Ok(Self::Reverse),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Strand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Self::Forward),
            "-" => Ok(Self::Reverse),
            _ => Err(()),
        }
    }
}

impl TryFrom<i8> for Strand {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Forward),
            -1 => Ok(Self::Reverse),
            _ => Err(()),
        }
    }
}

impl TryFrom<Orientation> for Strand {
    type Error = ();

    fn try_from(value: Orientation) -> Result<Self, Self::Error> {
        match value {
            Orientation::Forward => Ok(Self::Forward),
            Orientation::Reverse => Ok(Self::Reverse),
            Orientation::Dual => Err(()),
        }
    }
}

impl Default for Strand {
    fn default() -> Self {
        Self::Forward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strand_flip() {
        assert_eq!(*Strand::Forward.flip(), Strand::Reverse);
        assert_eq!(*Strand::Reverse.flip(), Strand::Forward);
    }

    #[test]
    fn test_strand_flipped() {
        assert_eq!(Strand::Forward.flipped(), Strand::Reverse);
        assert_eq!(Strand::Reverse.flipped(), Strand::Forward);
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

    #[test]
    fn test_strand_default() {
        assert_eq!(Strand::default(), Strand::Forward);
    }
}
