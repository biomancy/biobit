pub use alignment_segments::AlignmentSegments;
pub use builder::ReaderBuilder;
pub use reader::Reader;
pub use transform::SegmentedAlignment;

mod alignment_segments;
mod indexed_reader;
mod query;
mod reader;
pub mod transform;

mod builder;
pub mod strdeductor;
