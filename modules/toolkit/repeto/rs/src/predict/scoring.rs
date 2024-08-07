use derive_getters::Dissolve;

use biobit_alignment_rs::pairwise::scoring::Score;

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
