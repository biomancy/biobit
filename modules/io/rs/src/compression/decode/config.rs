use super::algorithm::Algorithm;
use super::params;
use bitcode::{Decode, Encode};
use eyre::{Result, bail};
use std::path::Path;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Config {
    RawBytes(Algorithm), // Directly uncompressed raw bytes as-is
    Gzip,                // GZIP container
    Bgzf(params::Bgzf),  // BGZF container
}

impl Default for Config {
    fn default() -> Self {
        Self::UNCOMPRESSED
    }
}

impl Config {
    pub const UNCOMPRESSED: Config = Config::RawBytes(Algorithm::None);

    pub fn infer_from_path(path: impl AsRef<Path>) -> Self {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "gz" | "gzip" => Config::Gzip,
                "bgz" | "bgzf" => Config::Bgzf(Default::default()),
                _ => Config::UNCOMPRESSED,
            })
            .unwrap_or(Config::UNCOMPRESSED)
    }

    pub fn infer_from_nickname(name: &str) -> Result<Self> {
        Ok(match name {
            "gz" | "gzip" => Config::Gzip,
            "bgz" | "bgzip" | "bgzf" => Config::Bgzf(Default::default()),
            "raw" | "none" => Config::UNCOMPRESSED,
            _ => bail!("Unknown compression format: {}", name),
        })
    }
}
