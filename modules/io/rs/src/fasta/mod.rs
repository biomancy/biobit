mod indexed_reader;
mod reader;
mod record;
pub mod validate;
mod writer;

pub use indexed_reader::{IndexedReader, IndexedReaderMutOp};
pub use reader::Reader;
pub use record::{Record, RecordMutOp, RecordOp};
use std::num::NonZeroUsize;
pub use writer::Writer;

pub const DEFAULT_LINE_WIDTH: NonZeroUsize = NonZeroUsize::new(80).unwrap();
