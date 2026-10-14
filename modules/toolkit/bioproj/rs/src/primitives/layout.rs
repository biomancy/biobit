use super::{MatesOrientation, Strandedness};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// An explicit read layout used to interpret sequencing observations.
///
/// Strandedness describes single reads or the first read of an inward pair;
/// the second read has the opposite orientation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Layout {
    /// Each source molecule is sequenced once in one direction.
    Single { strandedness: Strandedness },
    /// Each source molecule is sequenced twice with a known mates orientation.
    Paired {
        strandedness: Strandedness,
        orientation: MatesOrientation,
    },
}

impl Display for Layout {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single { strandedness } => write!(formatter, "Single({strandedness})"),
            Self::Paired {
                strandedness,
                orientation,
            } => write!(formatter, "Paired({orientation}, {strandedness})"),
        }
    }
}
