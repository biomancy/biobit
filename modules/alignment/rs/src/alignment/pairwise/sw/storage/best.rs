use crate::analysis::alignment::pairwise::scoring;
use crate::analysis::alignment::pairwise::sw::algo::{BestorientationTracer, GapTracer, Tracer};
use crate::analysis::alignment::pairwise::sw::storage::AlignmentSeed;

use super::Storage;

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
}

impl<S: scoring::Score> Default for Best<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: scoring::Score> BestorientationTracer for Best<S> {
    type Score = S;

    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, score: Self::Score) {
        if score > self.best.score {
            self.best.row = row;
            self.best.col = col;
            self.best.score = score;
        }
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, score: Self::Score) {
        if score > self.best.score {
            self.best.row = row;
            self.best.col = col;
            self.best.score = score;
        }
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: Self::Score) {
        if score > self.best.score {
            self.best.row = row;
            self.best.col = col;
            self.best.score = score;
        }
    }
}

impl<S: scoring::Score> GapTracer for Best<S> {
    type Score = S;
}

impl<S: scoring::Score> Tracer for Best<S> {
    type Score = S;
}

impl<S: scoring::Score> Storage for Best<S> {
    fn reset(&mut self, newrows: usize, newcols: usize) {
        self.best = AlignmentSeed {
            row: newrows + 1,
            col: newcols + 1,
            score: S::min_value(),
        }
    }

    #[inline(always)]
    fn finalize(&mut self) -> Vec<AlignmentSeed<S>> {
        if self.best.score > S::min_value() {
            vec![self.best]
        } else {
            vec![]
        }
    }
}
