pub use delegate::Delegate;

mod delegate;
pub mod gaps;
pub mod symbols;

pub trait ScoringScheme: gaps::Scorer + symbols::Scorer {}

pub fn default() -> Delegate<symbols::MatchMismatch, gaps::Affine> {
    Delegate {
        symbols: symbols::MatchMismatch { same: 1, diff: -2 },
        gaps: gaps::Affine { open: -5, extend: -1 },
    }
}

pub fn compose<S: symbols::Scorer, G: gaps::Scorer>(symbols: S, gaps: G) -> Delegate<S, G> {
    Delegate { symbols, gaps }
}