#[allow(clippy::module_inception)]
mod codec;
mod dna;

pub use codec::{BitComplement, Bits, Codec};
pub use dna::Dna;
