use std::fmt::Display;

/// Orientation of reads in a paired-end library. For now, just a placeholder with a single variant.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(i8)]
pub enum MatesOrientation {
    #[default]
    Inward = 1,
}

impl MatesOrientation {
    /// Returns the symbol representation of the mates orientation.
    pub fn symbol(&self) -> &'static str {
        match self {
            MatesOrientation::Inward => "I",
        }
    }
}

impl Display for MatesOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
