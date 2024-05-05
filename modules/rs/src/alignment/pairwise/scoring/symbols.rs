use std::marker::PhantomData;

use crate::alignment::pairwise::scoring::Score;

pub trait Scorer {
    type Score: Score;
    type Symbol;

    fn score(
        &self,
        seq1pos: usize,
        s1: &Self::Symbol,
        seq2pos: usize,
        s2: &Self::Symbol,
    ) -> Self::Score;
}

pub trait PosInvariantScorer {
    type SymScore: Score;
    type Symbol;

    fn score(&self, s1: &Self::Symbol, s2: &Self::Symbol) -> Self::SymScore;
}

impl<T: PosInvariantScorer> Scorer for T {
    type Score = <Self as PosInvariantScorer>::SymScore;
    type Symbol = <Self as PosInvariantScorer>::Symbol;

    #[inline(always)]
    fn score(&self, _: usize, s1: &Self::Symbol, _: usize, s2: &Self::Symbol) -> Self::Score {
        self.score(s1, s2)
    }
}

pub struct Equality<S: Score, Symbol> {
    pub equal: S,
    pub different: S,
    _phantom: PhantomData<Symbol>,
}

impl<S: Score, Symbol: PartialEq> PosInvariantScorer for Equality<S, Symbol> {
    type SymScore = S;
    type Symbol = Symbol;

    #[inline(always)]
    fn score(&self, a: &Self::Symbol, b: &Self::Symbol) -> Self::SymScore {
        if a == b {
            self.equal
        } else {
            self.different
        }
    }
}

impl<S: Score, Symbol: PartialEq> Equality<S, Symbol> {
    pub fn new(equal: S, different: S) -> Self {
        Self {
            equal,
            different,
            _phantom: Default::default(),
        }
    }
}
