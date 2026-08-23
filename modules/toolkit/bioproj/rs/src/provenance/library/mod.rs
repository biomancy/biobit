//! Concrete, acquisition-ready library preparations.

pub mod p5p7;
mod root;
pub mod strandedness;
mod validate;

pub use root::{Library, LibraryId, LibraryIdRef, LibraryKind};
pub(crate) use validate::validate;
