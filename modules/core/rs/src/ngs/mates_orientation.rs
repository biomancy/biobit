use std::fmt::Display;

/// Orientation of reads in a paired-end library. For now, just a placeholder with a single variant.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(i8)]
pub enum MatesOrientation {
    #[default]
    Inward = 1,
}

impl Display for MatesOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MatesOrientation::Inward => write!(f, "I"),
        }
    }
}
