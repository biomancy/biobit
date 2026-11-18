pub use all_optimal::AllOptimal;
pub use best::Best;

use super::Tracer;
use crate::pairwise::scoring;

mod all_optimal;
mod best;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AlignmentSeed<S: scoring::Score> {
    pub row: usize,
    pub col: usize,
    pub score: S,
}

pub trait EndpointTracker: Tracer {
    fn finalize(&mut self) -> Vec<AlignmentSeed<Self::Score>>;
}
