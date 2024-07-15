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

impl Display for Strandedness {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Strandedness::Forward => write!(f, "F"),
            Strandedness::Reverse => write!(f, "R"),
            Strandedness::Unstranded => write!(f, "U"),
        }
    }
}
