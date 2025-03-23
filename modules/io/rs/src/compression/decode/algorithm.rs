use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Algorithm {
    None,    // No compression
    Deflate, // DEFLATE compression
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::None
    }
}
