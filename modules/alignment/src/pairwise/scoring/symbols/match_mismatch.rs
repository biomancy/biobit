use super::{EquivClassifier, EquivType, PosInvariantScorer, Score, Symbol};

pub struct MatchMismatch {
    pub same: Score,
    pub diff: Score,
}

impl EquivClassifier for MatchMismatch {
    #[inline(always)]
    fn classify(&self, s1: Symbol, s2: Symbol) -> EquivType {
        match s1 == s2 {
            true => EquivType::Match,
            false => EquivType::Mismatch
        }
    }
}

impl PosInvariantScorer for MatchMismatch {
    #[inline(always)]
    fn score(&self, a: Symbol, b: Symbol) -> Score {
        if a == b { self.same } else { self.diff }
    }
}

