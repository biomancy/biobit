use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Strand specificity of an RNA-derived library.
///
/// This describes the relationship between a library preparation and its
/// source RNA. An acquisition's read layout determines how that relationship
/// is observed in individual reads.
///
/// The recommended way to represent unreported strand specificity is `Option<Strandedness>`.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[repr(i8)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum Strandedness {
    /// The library preserves the forward orientation of the source RNA.
    Forward = 1,
    /// The library preserves the reverse orientation of the source RNA.
    Reverse = -1,
    /// The library does not preserve source-RNA orientation.
    #[default]
    Unstranded = 0,
}

impl Strandedness {
    /// Returns the full name of this strandedness.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Reverse => "Reverse",
            Self::Unstranded => "Unstranded",
        }
    }
}

impl Display for Strandedness {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Strandedness;

    #[cfg(feature = "bitcode")]
    #[test]
    fn bitcode_roundtrip() {
        for value in [
            Strandedness::Forward,
            Strandedness::Reverse,
            Strandedness::Unstranded,
        ] {
            assert_eq!(
                bitcode::decode::<Strandedness>(&bitcode::encode(&value)).unwrap(),
                value
            );
        }
    }

    #[test]
    fn rejects_unknown_and_serializes_unstranded() {
        assert_eq!(Strandedness::default(), Strandedness::Unstranded);
        assert!(serde_json::from_str::<Strandedness>(r#""Unknown""#).is_err());
        assert_eq!(
            serde_json::to_string(&Strandedness::Unstranded).unwrap(),
            r#""Unstranded""#
        );
    }
}
