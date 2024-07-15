pub use alignment_segments::AlignmentSegments;
pub use reader::{Reader, ReaderBuilder};

mod alignment_segments;
mod indexed_reader;
mod query;
mod reader;
pub mod transform;

pub mod strdeductor;
