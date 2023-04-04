pub use match_mismatch::MatchMismatch;
pub use complementarity::Complementarity;

pub use super::super::{Score, Symbol};

mod match_mismatch;
mod matrix;
mod complementarity;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum EquivType {
    Match,
    Mismatch,
    Equivalent,
}

pub trait EquivClassifier {
    fn classify(&self, s1: Symbol, s2: Symbol) -> EquivType;
}

pub trait Scorer: EquivClassifier {
    fn score(&self, seq1pos: usize, s1: Symbol, seq2pos: usize, s2: Symbol) -> Score;
}

pub trait PosInvariantScorer: EquivClassifier {
    fn score(&self, s1: Symbol, s2: Symbol) -> Score;
}

impl<T: PosInvariantScorer> Scorer for T {
    #[inline(always)]
    fn score(&self, _: usize, s1: Symbol, _: usize, s2: Symbol) -> Score {
        self.score(s1, s2)
    }
}
