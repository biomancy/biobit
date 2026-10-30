pub use allopt::AllOptimal;
pub use best::Best;

use crate::pairwise::scoring;
use crate::pairwise::sw::algo::Tracer;

mod allopt;
mod best;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AlignmentSeed<S: scoring::Score> {
    pub row: usize,
    pub col: usize,
    pub score: S,
}

pub trait Storage: Tracer {
    /// Starts a scan with `(actual_len, used_len)` for each sequence.
    /// Each used length bounds the prefix visited along that sequence.
    fn reset(&mut self, seq1: (usize, usize), seq2: (usize, usize));

    fn finalize(&mut self) -> Vec<AlignmentSeed<<Self as Tracer>::Score>>;
}
