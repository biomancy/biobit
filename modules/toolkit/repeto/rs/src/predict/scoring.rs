use derive_getters::Dissolve;

use biobit_alignment_rs::pairwise::scoring::Score;

/// Scores for repeat prediction. Gap scores must satisfy `gap_open <= gap_extend <= 0`.
/// A gap of length `n` scores `gap_open + (n - 1) * gap_extend`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Dissolve)]
pub struct Scoring<S: Score> {
    pub complementary: S,
    pub mismatch: S,

    pub gap_open: S,
    pub gap_extend: S,
}

impl<S: Score> Default for Scoring<S> {
    fn default() -> Self {
        Scoring {
            complementary: S::one(),
            mismatch: S::zero() - (S::one() + S::one()),
            gap_open: S::zero() - (S::one() + S::one() + S::one() + S::one() + S::one()),
            gap_extend: S::zero() - S::one(),
        }
    }
}
