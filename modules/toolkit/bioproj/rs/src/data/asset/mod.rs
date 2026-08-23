//! Immutable stored data artifacts.

pub mod fastq;
mod root;

pub use root::{Asset, AssetId, AssetIdRef, AssetKind};

#[cfg(test)]
mod tests {
    use super::Asset;
    use super::fastq::{Fastq, FastqId};

    #[test]
    fn assets_are_independent_storage_records() {
        let asset = Asset::Fastq(
            Fastq::new(
                FastqId::new("AST1").unwrap(),
                "file:reads.fq.gz",
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        );

        let json = serde_json::to_string(&asset).unwrap();
        assert_eq!(
            json,
            r#"{"type":"Fastq","id":"AST1","location":"file:reads.fq.gz"}"#
        );
        assert_eq!(serde_json::from_str::<Asset>(&json).unwrap(), asset);
    }
}
