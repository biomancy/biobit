use bitcode::{Decode, Encode};
use eyre::{bail, Result};
use std::fmt::Display;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Algorithm {
    None,    // No compression
    Deflate, // DEFLATE compression
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::None => write!(f, "none"),
            Algorithm::Deflate => write!(f, "deflate"),
        }
    }
}

impl Algorithm {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "none" => Ok(Algorithm::None),
            "deflate" => Ok(Algorithm::Deflate),
            _ => bail!("Unknown compression algorithm: {}", s),
        }
    }
}
