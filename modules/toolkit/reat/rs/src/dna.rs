#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use eyre::Report;
use std::fmt::{Display, Formatter};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug, Default)]
pub enum Reference {
    A,
    C,
    G,
    T,
    #[default]
    N,
}

impl Reference {
    pub fn symbol(&self) -> &'static str {
        match self {
            Reference::A => "A",
            Reference::C => "C",
            Reference::G => "G",
            Reference::T => "T",
            Reference::N => "N",
        }
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl TryFrom<u8> for Reference {
    type Error = Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'A' | b'a' => Ok(Reference::A),
            b'C' | b'c' => Ok(Reference::C),
            b'G' | b'g' => Ok(Reference::G),
            b'T' | b't' => Ok(Reference::T),
            // All IUPAC ambiguity codes are treated as N
            // E.g., W, S, M, K, R, Y, B, D, H, V are all treated as N
            b'N' | b'n' => Ok(Reference::N),
            b'W' | b'w' | b'S' | b's' | b'M' | b'm' | b'K' | b'k' | b'R' | b'r' | b'Y' | b'y'
            | b'B' | b'b' | b'D' | b'd' | b'H' | b'h' | b'V' | b'v' => Ok(Reference::N),
            _ => Err(Report::msg(format!(
                "Invalid reference symbol: {}",
                value as char
            ))),
        }
    }
}

impl TryFrom<&str> for Reference {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A" | "a" => Ok(Reference::A),
            "C" | "c" => Ok(Reference::C),
            "G" | "g" => Ok(Reference::G),
            "T" | "t" => Ok(Reference::T),
            // All IUPAC ambiguity codes are treated as N
            // E.g., W, S, M, K, R, Y, B, D, H, V are all treated as N
            "N" | "n" => Ok(Reference::N),
            "W" | "w" | "S" | "s" | "M" | "m" | "K" | "k" | "R" | "r" | "Y" | "y" | "B" | "b"
            | "D" | "d" | "H" | "h" | "V" | "v" => Ok(Reference::N),
            _ => Err(Report::msg(format!("Invalid reference symbol: {}", value))),
        }
    }
}

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug, Default)]
pub enum Observed {
    A,
    C,
    G,
    T,
    #[default]
    N,
    Deletion,
}

impl Observed {
    pub fn symbol(&self) -> &'static str {
        match self {
            Observed::A => "A",
            Observed::C => "C",
            Observed::G => "G",
            Observed::T => "T",
            Observed::N => "N",
            Observed::Deletion => "D",
        }
    }
}

impl Display for Observed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl From<Reference> for Observed {
    fn from(reference: Reference) -> Self {
        match reference {
            Reference::A => Observed::A,
            Reference::C => Observed::C,
            Reference::G => Observed::G,
            Reference::T => Observed::T,
            Reference::N => Observed::N,
        }
    }
}

impl From<u8> for Observed {
    fn from(symbol: u8) -> Self {
        match symbol {
            b'A' | b'a' => Observed::A,
            b'C' | b'c' => Observed::C,
            b'G' | b'g' => Observed::G,
            b'T' | b't' => Observed::T,
            _ => Observed::N,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_from_sequence_symbol() {
        assert_eq!(Reference::try_from(b'A').unwrap(), Reference::A);
        assert_eq!(Reference::try_from(b'c').unwrap(), Reference::C);
        assert_eq!(Reference::N.symbol(), "N");
        assert!(Reference::try_from(b'X').is_err());
    }

    #[test]
    fn observed_from_sequence_symbol() {
        assert_eq!(Observed::from(b'G'), Observed::G);
        assert_eq!(Observed::from(b't'), Observed::T);
        assert_eq!(Observed::from(b'.'), Observed::N);
        assert_eq!(Observed::Deletion.symbol(), "D");
    }

    #[test]
    fn reference_can_be_observed() {
        assert_eq!(Observed::from(Reference::A), Observed::A);
        assert_eq!(Observed::from(Reference::N), Observed::N);
    }
}
