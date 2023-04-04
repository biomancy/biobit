pub use allopt::AllOptimal;
pub use best::Best;

use super::algo::{BestDirectionTracer, GapTracer, Tracer};
use super::Score;

mod allopt;
mod best;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AlignmentSeed {
    pub row: usize,
    pub col: usize,
    pub score: Score,
}

pub trait Storage: Tracer {
    fn reset(&mut self, newrows: usize, newcols: usize);

    fn finalize(&mut self) -> Vec<AlignmentSeed>;
}
