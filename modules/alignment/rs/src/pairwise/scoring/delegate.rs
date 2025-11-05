use std::marker::PhantomData;

use crate::pairwise::scoring::{Score, equiv, gaps, symbols};

pub struct Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    pub symbols: S,
    pub gaps: G,
    pub equiv: E,
    symbol: PhantomData<Symbol>,
    score: PhantomData<ScoreType>,
}

impl<ScoreType, Symbol, S, G, E> Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    pub fn new(symbols: S, gaps: G, equiv: E) -> Self {
        Delegate {
            symbols,
            gaps,
            equiv,
            symbol: Default::default(),
            score: Default::default(),
        }
    }
}

impl<ScoreType, Symbol, S, G, E> gaps::Scorer for Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    type Score = ScoreType;

    #[inline(always)]
    fn seq1_gap_open(&self, pos: usize) -> Self::Score {
        self.gaps.seq1_gap_open(pos)
    }

    #[inline(always)]
    fn seq1_gap_extend(&self, pos: usize) -> Self::Score {
        self.gaps.seq1_gap_extend(pos)
    }

    #[inline(always)]
    fn seq2_gap_open(&self, pos: usize) -> Self::Score {
        self.gaps.seq2_gap_open(pos)
    }

    #[inline(always)]
    fn seq2_gap_extend(&self, pos: usize) -> Self::Score {
        self.gaps.seq2_gap_extend(pos)
    }
}

impl<ScoreType, Symbol, S, G, E> equiv::Classifier for Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    type Symbol = Symbol;

    #[inline(always)]
    fn classify(&self, s1: &Self::Symbol, s2: &Self::Symbol) -> equiv::Type {
        self.equiv.classify(s1, s2)
    }
}

impl<ScoreType, Symbol, S, G, E> symbols::Scorer for Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    type Score = ScoreType;
    type Symbol = Symbol;

    #[inline(always)]
    fn score(&self, posa: usize, a: &Self::Symbol, posb: usize, b: &Self::Symbol) -> Self::Score {
        self.symbols.score(posa, a, posb, b)
    }
}

impl<ScoreType, Symbol, S, G, E> super::Scheme for Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    type Score = ScoreType;
    type Symbol = Symbol;
}
