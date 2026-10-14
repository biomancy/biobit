use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The relative orientation of the two reads in paired-end sequencing.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[repr(i8)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
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

#[cfg(all(test, feature = "bitcode"))]
mod tests {
    use super::MatesOrientation;

    #[test]
    fn bitcode_roundtrip() {
        let value = MatesOrientation::Inward;
        assert_eq!(
            bitcode::decode::<MatesOrientation>(&bitcode::encode(&value)).unwrap(),
            value
        );
    }
}
