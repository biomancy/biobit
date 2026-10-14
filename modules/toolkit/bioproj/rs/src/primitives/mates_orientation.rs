use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The relative orientation of the two reads in paired-end sequencing.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[repr(i8)]
pub enum MatesOrientation {
    /// The two reads face toward one another along the source molecule.
    #[default]
    Inward = 1,
}

impl MatesOrientation {
    /// Returns the full name of this mates orientation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Inward => "Inward",
        }
    }
}

impl Display for MatesOrientation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
