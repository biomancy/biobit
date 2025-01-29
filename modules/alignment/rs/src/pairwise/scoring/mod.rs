pub use delegate::Delegate;

pub use crate::Score;

mod delegate;
pub mod equiv;
pub mod gaps;
pub mod symbols;

pub trait Scheme:
    gaps::Scorer<Score = <Self as Scheme>::Score>
    + symbols::Scorer<Score = <Self as Scheme>::Score, Symbol = <Self as Scheme>::Symbol>
    + equiv::Classifier<Symbol = <Self as Scheme>::Symbol>
{
    type Score: Score;
    type Symbol;
}

pub fn compose<ScoreType, Symbol, S, G, E>(
    symbols: S,
    gaps: G,
    equiv: E,
) -> Delegate<ScoreType, Symbol, S, G, E>
where
    ScoreType: Score,
    S: symbols::Scorer<Symbol = Symbol, Score = ScoreType>,
    G: gaps::Scorer<Score = ScoreType>,
    E: equiv::Classifier<Symbol = Symbol>,
{
    Delegate::new(symbols, gaps, equiv)
}
