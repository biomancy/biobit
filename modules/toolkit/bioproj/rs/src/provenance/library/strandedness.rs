use serde::{Deserialize, Serialize};

/// Strand specificity of an RNA-derived library.
///
/// This describes the relationship between a library preparation and its
/// source RNA. An assay's acquisition layout determines how that relationship
/// is observed in individual reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Strandedness {
    /// The library's strand specificity was not reported.
    Unknown,
    /// The library preserves the forward orientation of the source RNA.
    Forward,
    /// The library preserves the reverse orientation of the source RNA.
    Reverse,
    /// The library does not preserve source-RNA orientation.
    Unstranded,
}

#[cfg(test)]
mod tests {
    use super::Strandedness;

    #[test]
    fn serializes_unknown_distinctly_from_unstranded() {
        assert_eq!(
            serde_json::to_string(&Strandedness::Unknown).unwrap(),
            r#""Unknown""#
        );
        assert_eq!(
            serde_json::to_string(&Strandedness::Unstranded).unwrap(),
            r#""Unstranded""#
        );
    }
}
