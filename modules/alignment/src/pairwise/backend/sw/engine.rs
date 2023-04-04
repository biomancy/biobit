use itertools::Itertools;

use super::{
    algo::{BestDirectionTracer, FullScan, GapTracer, Tracer},
    Score,
};
use super::storage::{AllOptimal, Storage};
use super::super::super::{Alignable, alignment, local};
use super::super::super::scoring::ScoringScheme;
use super::traceback::TraceMat;

struct Tracers<S: Storage, T: TraceMat> {
    pub storage: S,
    pub tracemat: T,
}


impl<S: Storage, T: TraceMat> BestDirectionTracer for Tracers<S, T> {
    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, score: Score) {
        self.storage.gap_row(row, col, score);
        self.tracemat.gap_row(row, col, score);
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, score: Score) {
        self.storage.gap_col(row, col, score);
        self.tracemat.gap_col(row, col, score);
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: Score) {
        self.storage.equivalent(row, col, score);
        self.tracemat.equivalent(row, col, score);
    }

    #[inline(always)]
    fn none(&mut self, row: usize, col: usize) {
        self.storage.none(row, col);
        self.tracemat.none(row, col);
    }
}

impl<S: Storage, T: TraceMat> GapTracer for Tracers<S, T> {
    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, score: Score) {
        self.storage.row_gap_open(row, col, score);
        self.tracemat.row_gap_open(row, col, score);
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, score: Score) {
        self.storage.row_gap_extend(row, col, score);
        self.tracemat.row_gap_extend(row, col, score);
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, score: Score) {
        self.storage.col_gap_open(row, col, score);
        self.tracemat.col_gap_open(row, col, score);
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, score: Score) {
        self.storage.col_gap_extend(row, col, score);
        self.tracemat.col_gap_extend(row, col, score);
    }
}

impl<S: Storage, T: TraceMat> Tracer for Tracers<S, T> {
    #[inline(always)]
    fn first_col_start(&mut self) {
        self.storage.first_col_start();
        self.tracemat.first_col_start();
    }

    #[inline(always)]
    fn first_col_end(&mut self) {
        self.storage.first_col_end();
        self.tracemat.first_col_end();
    }

    #[inline(always)]
    fn col_start(&mut self, col: usize) {
        self.storage.col_start(col);
        self.tracemat.col_start(col);
    }

    #[inline(always)]
    fn col_end(&mut self, col: usize) {
        self.storage.col_end(col);
        self.tracemat.col_end(col);
    }
}


pub struct Engine<S: Storage, T: TraceMat, SF: ScoringScheme> {
    algo: FullScan,
    scoring: SF,
    tracers: Tracers<S, T>,
}


impl<S: Storage, T: TraceMat, SF: ScoringScheme> Engine<S, T, SF> {
    pub fn new(storage: S, tracemat: T, scoring: SF) -> Self {
        let algo = FullScan::new(0);
        Self { algo, scoring, tracers: Tracers { storage, tracemat } }
    }

    fn run<S1: Alignable, S2: Alignable>(&mut self, seq1: &S1, seq2: &S2) -> Vec<local::Alignment> {
        if seq1.len() == 0 || seq2.len() == 0 {
            return vec![];
        }

        self.tracers.storage.reset(seq1.len(), seq2.len());
        self.tracers.tracemat.reset(seq1.len(), seq2.len());

        self.algo.scan_all(seq1, seq2, &mut self.scoring, &mut self.tracers);

        self.tracers.storage.finalize().into_iter().map(|x| {
            let trace = self.tracers.tracemat.trace(x.row, x.col).unwrap();
            debug_assert_eq!(trace.seq1.end, x.row + 1);
            debug_assert_eq!(trace.seq2.end, x.col + 1);
            let ops = alignment::utils::disambiguate(
                trace.ops, &self.scoring,
                seq1, trace.seq1.start,
                seq2, trace.seq2.start,
            );

            local::Alignment {
                score: x.score,
                steps: ops,
                seq1: trace.seq1,
                seq2: trace.seq2,
            }
        }).collect()
    }
}

impl<S1: Alignable, S2: Alignable, S: Storage, T: TraceMat, SF: ScoringScheme> local::Aligner<S1, S2, SF> for Engine<S, T, SF> {
    fn with_scoring(&mut self, scoring: SF) {
        self.scoring = scoring
    }

    fn align(&mut self, seq1: &S1, seq2: &S2) -> Result<local::Alignment, ()> {
        self.run(seq1, seq2).into_iter().max_by_key(|x| x.score).ok_or(())
    }
}

impl<T: TraceMat, SF: ScoringScheme> local::MultiAlignerConfig<SF> for Engine<AllOptimal, T, SF> {
    fn set_scoring(&mut self, scoring: SF) {
        self.scoring = scoring
    }

    fn set_min_score_thr(&mut self, thr: Score) {
        self.tracers.storage.minscore = thr;
    }
}

impl<S1: Alignable, S2: Alignable, T: TraceMat, SF: ScoringScheme> local::MultiAligner<S1, S2, SF> for Engine<AllOptimal, T, SF> {
    fn align(&mut self, seq1: &S1, seq2: &S2, saveto: &mut Vec<local::Alignment>) {
        let mut results = self.run(seq1, seq2);

        // Collapse overlapping paths & save results
        results.sort_by_key(|x| x.score);
        let startind = saveto.len();
        for item in results.into_iter().rev() {
            let intersects = saveto[startind..].iter().any(|x| x.intersects(&item));
            if !intersects {
                saveto.push(item);
            }
        }
    }

    fn uptriangle(&mut self, seq1: &S1, seq2: &S2, offset: usize, saveto: &mut Vec<local::Alignment>) {
        self.tracers.storage.reset(seq1.len(), seq2.len());
        self.tracers.tracemat.reset(seq1.len(), seq2.len());

        self.algo.scan_up_triangle(seq1, seq2, offset, &mut self.scoring, &mut self.tracers);

        let results = self.tracers.storage.finalize().into_iter().map(|x| {
            let trace = self.tracers.tracemat.trace(x.row, x.col).unwrap();
            debug_assert_eq!(trace.seq1.end, x.row + 1);
            debug_assert_eq!(trace.seq2.end, x.col + 1);
            let ops = alignment::utils::disambiguate(
                trace.ops, &self.scoring,
                seq1, trace.seq1.start,
                seq2, trace.seq2.start,
            );

            local::Alignment {
                score: x.score,
                steps: ops,
                seq1: trace.seq1,
                seq2: trace.seq2,
            }
        }).sorted_by_key(|x| x.score);

        // Collapse overlapping paths & save results
        let startind = saveto.len();
        for item in results.into_iter().rev() {
            let intersects = saveto[startind..].iter().any(|x| x.intersects(&item));
            if !intersects {
                saveto.push(item);
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::{storage, traceback};
    use super::super::super::super::scoring;

    #[test]
    fn sw_local_best() {
        let mut engine = Engine::new(
            storage::Best::new(),
            traceback::TraceMatrix::new(),
            scoring::default(),
        );

        local::test_suite::best::test_all(&mut engine);
    }

    #[test]
    fn sw_local_alloptimal() {
        let mut engine = Engine::new(
            storage::AllOptimal::new(),
            traceback::TraceMatrix::new(),
            scoring::default(),
        );

        local::test_suite::alloptimal::test_all(&mut engine);
    }
}
