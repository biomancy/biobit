pub use local::FullScan;

use super::super::super::{Alignable, Score};
use super::super::super::scoring::ScoringScheme;

mod local;

// All smith-waterman (sw) algorithms run column-by-column and notify of the
// choices other pieces of the algorithm

pub trait BestDirectionTracer {
    fn gap_row(&mut self, row: usize, col: usize, score: Score) {}
    fn gap_col(&mut self, row: usize, col: usize, score: Score) {}
    fn equivalent(&mut self, row: usize, col: usize, score: Score) {}
    fn none(&mut self, row: usize, col: usize) {}
}

pub trait GapTracer {
    fn row_gap_open(&mut self, row: usize, col: usize, score: Score) {}
    fn row_gap_extend(&mut self, row: usize, col: usize, score: Score) {}

    fn col_gap_open(&mut self, row: usize, col: usize, score: Score) {}
    fn col_gap_extend(&mut self, row: usize, col: usize, score: Score) {}
}

pub trait Tracer: BestDirectionTracer + GapTracer {
    fn first_col_start(&mut self) {}
    fn first_col_end(&mut self) {}

    fn col_start(&mut self, col: usize) {}
    fn col_end(&mut self, col: usize) {}
}


// use super::super::storage::Storage;
// use super::super::super::scoring::ScoringScheme;
// use super::super::traceback::Tracer;
//
// pub struct Context<S: Storage, SF: ScoringScheme, T: Tracer> {
//     pub storage: S,
//     pub scoring: SF,
//     pub tracer: T,
// }
