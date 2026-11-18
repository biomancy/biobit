use super::{AlignmentSeed, EndpointTracker};
use crate::pairwise::dp::{
    Cursor,
    layout::Layout,
    tracer::{CellScores, Trace, Tracer},
};
use crate::pairwise::scoring;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Best<S: scoring::Score> {
    best: AlignmentSeed<S>,
}

impl<S: scoring::Score> Best<S> {
    pub fn new() -> Self {
        Self {
            best: AlignmentSeed {
                row: 0,
                col: 0,
                score: S::min_value(),
            },
        }
    }

    #[inline]
    fn consider(best: &mut AlignmentSeed<S>, col: usize, row: usize, score: S) {
        if score > S::zero() && score > best.score {
            *best = AlignmentSeed { row, col, score };
        }
    }
}

impl<S: scoring::Score> Default for Best<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: scoring::Score> Tracer for Best<S> {
    type Score = S;

    fn initialize(&mut self, _: &impl Layout) {
        *self = Self::new();
    }

    #[inline]
    fn chunk<const N: usize>(
        &mut self,
        _: &impl Layout,
        pointer: &Cursor,
        scores: &[CellScores<S>; N],
        _: &[Trace; N],
    ) {
        let mut best = self.best;
        for (i, score) in scores.iter().enumerate() {
            Self::consider(&mut best, pointer.col, pointer.row + i, score.best);
        }
        self.best = best;
    }

    #[inline]
    fn tail(&mut self, _: &impl Layout, pointer: &Cursor, scores: &[CellScores<S>], _: &[Trace]) {
        let mut best = self.best;
        for (i, score) in scores.iter().enumerate() {
            Self::consider(&mut best, pointer.col, pointer.row + i, score.best);
        }
        self.best = best;
    }
}

impl<S: scoring::Score> EndpointTracker for Best<S> {
    fn finalize(&mut self) -> Vec<AlignmentSeed<S>> {
        if self.best.score > S::min_value() {
            vec![self.best]
        } else {
            vec![]
        }
    }
}
