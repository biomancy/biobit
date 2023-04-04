pub use super::super::Score;

// Gap scoring function MUST be additive
// In other words, you can have whatever gapopen / gapextend scores you want as long as scores are additive (!)

pub trait Scorer {
    fn seq1_gap_open(&self, pos: usize) -> Score;
    fn seq1_gap_extend(&self, pos: usize) -> Score;

    fn seq2_gap_open(&self, pos: usize) -> Score;
    fn seq2_gap_extend(&self, pos: usize) -> Score;
}

pub trait PosInvariantScorer {
    fn gap_open(&self) -> Score;
    fn gap_extend(&self) -> Score;
}

impl<T: PosInvariantScorer> Scorer for T {
    #[inline(always)]
    fn seq1_gap_open(&self, _: usize) -> Score {
        self.gap_open()
    }

    #[inline(always)]
    fn seq1_gap_extend(&self, _: usize) -> Score {
        self.gap_extend()
    }

    #[inline(always)]
    fn seq2_gap_open(&self, _: usize) -> Score {
        self.gap_open()
    }

    #[inline(always)]
    fn seq2_gap_extend(&self, _: usize) -> Score {
        self.gap_extend()
    }
}

pub struct Affine {
    pub open: Score,
    pub extend: Score,
}

impl PosInvariantScorer for Affine {
    #[inline(always)]
    fn gap_open(&self) -> Score {
        self.open
    }

    #[inline(always)]
    fn gap_extend(&self) -> Score {
        self.extend
    }
}
