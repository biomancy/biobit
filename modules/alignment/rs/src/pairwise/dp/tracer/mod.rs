use crate::pairwise::{
    dp::{Cursor, layout::Layout},
    scoring,
};

mod endpoints;
mod trace_cell;
mod traceback;

pub use endpoints::{AlignmentSeed, AllOptimal, Best, EndpointTracker};
pub use trace_cell::{BestTrace, Trace};
pub use traceback::{TraceMatrix, Traceback, TracedAlignment};

/// Scores for one cell. A zero gap score means that gap is inactive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellScores<S> {
    pub best: S,
    pub row_gap: S,
    pub col_gap: S,
}

/// Receives one initialization per scan, including empty scans. The kernel visits
/// columns in ascending order. Each chunk holds consecutive rows, and chunks
/// within a column arrive in increasing row order. Every callback borrows the same layout for the duration of the scan.
pub trait Tracer {
    type Score: scoring::Score;

    /// Initializes the tracer for a new scan of the supplied layout.
    fn initialize(&mut self, layout: &impl Layout);

    fn col_start(&mut self, _layout: &impl Layout, _col: usize) {}
    fn col_end(&mut self, _layout: &impl Layout, _col: usize) {}

    /// Delivers a full chunk starting at `pointer`. Score and trace `i` are
    /// `i` rows and `i` dense indices past that position.
    fn chunk<const N: usize>(
        &mut self,
        layout: &impl Layout,
        pointer: &Cursor,
        scores: &[CellScores<Self::Score>; N],
        traces: &[Trace; N],
    );

    /// Delivers the nonempty remainder of a column. The trace slice contains
    /// `scores.len()` cells with the same indexing as a full chunk.
    fn tail(
        &mut self,
        layout: &impl Layout,
        pointer: &Cursor,
        scores: &[CellScores<Self::Score>],
        traces: &[Trace],
    );
}
