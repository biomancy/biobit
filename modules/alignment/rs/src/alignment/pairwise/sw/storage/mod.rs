pub use allopt::AllOptimal;
pub use best::Best;

use crate::analysis::alignment::pairwise::scoring;
use crate::analysis::alignment::pairwise::sw::algo::Tracer;

mod allopt;
mod best;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AlignmentSeed<S: scoring::Score> {
    pub row: usize,
    pub col: usize,
    pub score: S,
}

pub trait Storage: Tracer {
    fn reset(&mut self, newrows: usize, newcols: usize);

    fn finalize(&mut self) -> Vec<AlignmentSeed<<Self as Tracer>::Score>>;
}
