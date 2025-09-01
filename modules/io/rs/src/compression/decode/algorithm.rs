#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Algorithm {
    #[default]
    None, // No compression
    Deflate, // DEFLATE compression
}
