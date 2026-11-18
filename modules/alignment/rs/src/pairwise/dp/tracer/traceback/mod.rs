use std::ops::Range;

pub use matrix::TraceMatrix;

use super::Tracer;
use crate::pairwise::{Step, dp::layout::Layout};
use eyre::Result;

mod matrix;

pub struct TracedAlignment {
    pub ops: Vec<Step<u8>>,
    pub seq1: Range<usize>,
    pub seq2: Range<usize>,
}

pub trait Traceback: Tracer {
    /// Reconstructs the alignment ending at `(col, row)` using the layout of the last scan.
    fn trace(&self, layout: &impl Layout, col: usize, row: usize) -> Result<TracedAlignment>;
}
