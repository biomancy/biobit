use std::fmt::Display;

/// Strandedness of a sequencing library. Indicates the relationship between molecules in the
/// library and their source DNA/RNA strand. DNA-based libraries  are typically unstranded,
/// while RNA-based libraries can be either stranded or unstranded.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(i8)]
pub enum Strandedness {
    /// Each sequenced read matches the sequence of the source molecule.
    Forward = 1,
    /// Each sequenced read is the reverse complement of the source molecule.
    Reverse = -1,
    /// Each sequenced read can be either identical to the source molecule or its reverse complement.
    #[default]
    Unstranded = 0,
}

impl Strandedness {
    /// Returns the symbol representation of the strandedness.
    pub fn symbol(&self) -> &'static str {
        match self {
            Strandedness::Forward => "F",
            Strandedness::Reverse => "R",
            Strandedness::Unstranded => "U",
        }
    }
}

impl Display for Strandedness {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}
