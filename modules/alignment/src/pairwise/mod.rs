pub use self::alignment::{AlignmentOp, AlignmentStep};
pub use constraint::{ConstrainedPos, Constraint};

pub use super::{Alignable, Score, Symbol};

pub mod scoring;
mod constraint;
mod alignment;
pub mod local;
pub mod backend;

