use super::{EquivClassifier, EquivType, PosInvariantScorer, Score, Symbol};

pub struct Complementarity {
    pub complementary: Score,
    pub otherwise: Score,
}

impl Complementarity {
    #[inline(always)]
    fn is_complementary(&self, a: Symbol, b: Symbol) -> bool {
        match (a, b) {
            (b'A', b'T') | (b'T', b'A') |
            (b'A', b'U') | (b'U', b'A') |
            (b'G', b'C') | (b'C', b'G') |
            (b'G', b'U') | (b'U', b'G') => true,
            _ => false
        }
    }
}

impl EquivClassifier for Complementarity {
    #[inline(always)]
    fn classify(&self, s1: Symbol, s2: Symbol) -> EquivType {
        return if self.is_complementary(s1, s2) {
            EquivType::Match
        } else {
            EquivType::Mismatch
        };
    }
}

impl PosInvariantScorer for Complementarity {
    #[inline(always)]
    fn score(&self, a: Symbol, b: Symbol) -> Score {
        return if self.is_complementary(a, b) {
            self.complementary
        } else {
            self.otherwise
        };
    }
}

