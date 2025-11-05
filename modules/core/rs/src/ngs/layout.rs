use super::{mates_orientation::MatesOrientation, strandedness::Strandedness};
use std::fmt::{Display, Formatter};

// Inspired by Salmon: https://salmon.readthedocs.io/en/latest/library_type.html
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Layout {
    /// Each source molecule is sequenced only once (single-end sequencing) in one direction.
    Single { strandedness: Strandedness },
    /// Each source molecule is sequenced twice (paired-end sequencing) following a known orientation.
    /// Strandedness interpretation is orientation-dependent.
    Paired {
        strandedness: Strandedness,
        orientation: MatesOrientation,
    },
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Single { strandedness } => {
                write!(f, "Single({})", strandedness)
            }
            Layout::Paired {
                strandedness,
                orientation,
            } => {
                write!(f, "Paired({}, {})", orientation, strandedness)
            }
        }
    }
}
