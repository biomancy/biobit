use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Reference {
    A,
    C,
    G,
    T,
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

impl Default for Reference {
    fn default() -> Self {
        Reference::N
    }
}

impl From<u8> for Reference {
    fn from(symbol: u8) -> Self {
        match symbol {
            b'A' | b'a' => Reference::A,
            b'C' | b'c' => Reference::C,
            b'G' | b'g' => Reference::G,
            b'T' | b't' => Reference::T,
            _ => Reference::N,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Observed {
    A,
    C,
    G,
    T,
    N,
    Deletion,
    Insertion,
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
            Observed::Insertion => "I",
        }
    }
}

impl Display for Observed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl Default for Observed {
    fn default() -> Self {
        Observed::N
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
        assert_eq!(Reference::from(b'A'), Reference::A);
        assert_eq!(Reference::from(b'c'), Reference::C);
        assert_eq!(Reference::from(b'X'), Reference::N);
        assert_eq!(Reference::N.symbol(), "N");
    }

    #[test]
    fn observed_from_sequence_symbol() {
        assert_eq!(Observed::from(b'G'), Observed::G);
        assert_eq!(Observed::from(b't'), Observed::T);
        assert_eq!(Observed::from(b'.'), Observed::N);
        assert_eq!(Observed::Deletion.symbol(), "D");
        assert_eq!(Observed::Insertion.symbol(), "I");
    }

    #[test]
    fn reference_can_be_observed() {
        assert_eq!(Observed::from(Reference::A), Observed::A);
        assert_eq!(Observed::from(Reference::N), Observed::N);
    }
}
