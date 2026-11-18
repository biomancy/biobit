//! Dynamic programming for pairwise alignment.

pub use cursor::Cursor;
pub use engine::Engine;
pub use kernel::ChunkSize;
pub use scan::Scan;

mod cursor;
mod engine;
pub mod kernel;
pub mod layout;
mod scan;
pub mod tracer;
