pub use alignment_segments::AlignmentSegments;
pub use reader::{Reader, ReaderBuilder};
pub use traits::{AdaptersForIndexedBAM, IndexedBAM};

pub mod adapters;
mod alignment_segments;
mod indexed_reader;
mod query;
mod reader;
pub mod strdeductor;
mod traits;
