use std::marker::PhantomData;

use derive_getters::Dissolve;

use crate::pairwise::scoring;
use crate::pairwise::sw::{algo, storage, traceback};

#[derive(Dissolve)]
pub struct Tracers<S, Storage, TraceMat>
where
    S: scoring::Score,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    pub storage: Storage,
    pub tracemat: TraceMat,
    pub _phantom: PhantomData<S>,
}

impl<S, Storage, TraceMat> algo::BestOrientationTracer for Tracers<S, Storage, TraceMat>
where
    S: scoring::Score,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    type Score = S;

    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.gap_row(row, col, score.clone());
        self.tracemat.gap_row(row, col, score);
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.gap_col(row, col, score.clone());
        self.tracemat.gap_col(row, col, score);
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.equivalent(row, col, score.clone());
        self.tracemat.equivalent(row, col, score);
    }

    #[inline(always)]
    fn none(&mut self, row: usize, col: usize) {
        self.storage.none(row, col);
        self.tracemat.none(row, col);
    }
}

impl<S, Storage, TraceMat> algo::GapTracer for Tracers<S, Storage, TraceMat>
where
    S: scoring::Score,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    type Score = S;

    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.row_gap_open(row, col, score.clone());
        self.tracemat.row_gap_open(row, col, score);
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.row_gap_extend(row, col, score.clone());
        self.tracemat.row_gap_extend(row, col, score);
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.col_gap_open(row, col, score.clone());
        self.tracemat.col_gap_open(row, col, score);
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, score: Self::Score) {
        self.storage.col_gap_extend(row, col, score.clone());
        self.tracemat.col_gap_extend(row, col, score);
    }
}

impl<S, Storage, TraceMat> algo::Tracer for Tracers<S, Storage, TraceMat>
where
    S: scoring::Score,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    type Score = S;

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

impl<S, Storage, TraceMat> Tracers<S, Storage, TraceMat>
where
    S: scoring::Score,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    pub fn reset(&mut self, newrows: usize, newcols: usize) {
        self.storage.reset(newrows, newcols);
        self.tracemat.reset(newrows, newcols);
    }
}
