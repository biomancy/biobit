use std::fmt::Display;

/// Orientation of reads in a paired-end library.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(i8)]
pub enum MatesOrientation {
    Inward = 1,
    Outward = -1,
    Matching = 0,
}

impl Display for MatesOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MatesOrientation::Inward => write!(f, "I"),
            MatesOrientation::Outward => write!(f, "O"),
            MatesOrientation::Matching => write!(f, "M"),
        }
    }
}
