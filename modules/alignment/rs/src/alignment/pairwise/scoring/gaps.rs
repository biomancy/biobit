use crate::analysis::alignment::pairwise::scoring::Score;

// Gap scoring function MUST be additive
// We can have whatever gapopen / gapextend scores you want as long as it's context independent
pub trait Scorer {
    type Score: Score;

    fn seq1_gap_open(&self, pos: usize) -> Self::Score;
    fn seq1_gap_extend(&self, pos: usize) -> Self::Score;

    fn seq2_gap_open(&self, pos: usize) -> Self::Score;
    fn seq2_gap_extend(&self, pos: usize) -> Self::Score;
}

pub trait PosInvariantScorer {
    type GapScore: Score;

    fn gap_open(&self) -> Self::GapScore;
    fn gap_extend(&self) -> Self::GapScore;
}

impl<T: PosInvariantScorer> Scorer for T {
    type Score = <Self as PosInvariantScorer>::GapScore;

    #[inline(always)]
    fn seq1_gap_open(&self, _: usize) -> Self::Score {
        self.gap_open()
    }

    #[inline(always)]
    fn seq1_gap_extend(&self, _: usize) -> Self::Score {
        self.gap_extend()
    }

    #[inline(always)]
    fn seq2_gap_open(&self, _: usize) -> Self::Score {
        self.gap_open()
    }

    #[inline(always)]
    fn seq2_gap_extend(&self, _: usize) -> Self::Score {
        self.gap_extend()
    }
}

pub struct Affine<S: Score> {
    pub open: S,
    pub extend: S,
}

impl<S: Score> PosInvariantScorer for Affine<S> {
    type GapScore = S;

    #[inline(always)]
    fn gap_open(&self) -> Self::GapScore {
        self.open
    }

    #[inline(always)]
    fn gap_extend(&self) -> Self::GapScore {
        self.extend
    }
}
