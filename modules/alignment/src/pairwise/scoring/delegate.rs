use super::{gaps, symbols};
use super::super::{Score, Symbol};

pub struct Delegate<S: symbols::Scorer, G: gaps::Scorer> {
    pub symbols: S,
    pub gaps: G,
}

impl<S: symbols::Scorer, G: gaps::Scorer> gaps::Scorer for Delegate<S, G> {
    #[inline(always)]
    fn seq1_gap_open(&self, pos: usize) -> Score {
        self.gaps.seq1_gap_open(pos)
    }

    #[inline(always)]
    fn seq1_gap_extend(&self, pos: usize) -> Score {
        self.gaps.seq1_gap_extend(pos)
    }

    #[inline(always)]
    fn seq2_gap_open(&self, pos: usize) -> Score {
        self.gaps.seq2_gap_open(pos)
    }

    #[inline(always)]
    fn seq2_gap_extend(&self, pos: usize) -> Score {
        self.gaps.seq2_gap_extend(pos)
    }
}

impl<S: symbols::Scorer, G: gaps::Scorer> symbols::EquivClassifier for Delegate<S, G> {
    #[inline(always)]
    fn classify(&self, s1: Symbol, s2: Symbol) -> symbols::EquivType {
        self.symbols.classify(s1, s2)
    }
}

impl<S: symbols::Scorer, G: gaps::Scorer> symbols::Scorer for Delegate<S, G> {
    #[inline(always)]
    fn score(&self, posa: usize, a: Symbol, posb: usize, b: Symbol) -> Score {
        self.symbols.score(posa, a, posb, b)
    }
}

impl<S: symbols::Scorer, G: gaps::Scorer> super::ScoringScheme for Delegate<S, G> {}
