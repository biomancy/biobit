use super::{AlignmentSeed, BestDirectionTracer, GapTracer, Score, Storage, Tracer};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Best {
    best: AlignmentSeed,
}

impl Best {
    pub fn new() -> Self {
        Self { best: AlignmentSeed { row: 0, col: 0, score: Score::MIN } }
    }
}

impl Tracer for Best {}

impl GapTracer for Best {}

impl BestDirectionTracer for Best {
    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, score: Score) {
        if score > self.best.score {
            self.best.row = row;
            self.best.col = col;
            self.best.score = score;
        }
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, score: Score) {
        if score > self.best.score {
            self.best.row = row;
            self.best.col = col;
            self.best.score = score;
        }
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: Score) {
        if score > self.best.score {
            self.best.row = row;
            self.best.col = col;
            self.best.score = score;
        }
    }
}


impl Storage for Best {
    fn reset(&mut self, newrows: usize, newcols: usize) {
        self.best = AlignmentSeed { row: newrows + 1, col: newcols + 1, score: Score::MIN }
    }

    #[inline(always)]
    fn finalize(&mut self) -> Vec<AlignmentSeed> {
        if self.best.score > Score::MIN {
            vec![self.best]
        } else {
            vec![]
        }
    }
}