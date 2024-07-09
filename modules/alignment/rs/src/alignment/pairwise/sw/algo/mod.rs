pub use local::FullScan;

use crate::alignment::pairwise::scoring;

mod local;

// All smith-waterman (sw) algorithms run column-by-column and
// notify other pieces of the algorithm about each step

#[allow(unused_variables)]
pub trait BestorientationTracer {
    type Score: scoring::Score;

    fn gap_row(&mut self, row: usize, col: usize, score: Self::Score) {}
    fn gap_col(&mut self, row: usize, col: usize, score: Self::Score) {}
    fn equivalent(&mut self, row: usize, col: usize, score: Self::Score) {}
    fn none(&mut self, row: usize, col: usize) {}
}

#[allow(unused_variables)]
pub trait GapTracer {
    type Score: scoring::Score;

    fn row_gap_open(&mut self, row: usize, col: usize, score: Self::Score) {}
    fn row_gap_extend(&mut self, row: usize, col: usize, score: Self::Score) {}

    fn col_gap_open(&mut self, row: usize, col: usize, score: Self::Score) {}
    fn col_gap_extend(&mut self, row: usize, col: usize, score: Self::Score) {}
}

#[allow(unused_variables)]
pub trait Tracer:
    BestorientationTracer<Score = <Self as Tracer>::Score> + GapTracer<Score = <Self as Tracer>::Score>
{
    type Score: scoring::Score;

    fn first_col_start(&mut self) {}
    fn first_col_end(&mut self) {}

    fn col_start(&mut self, col: usize) {}
    fn col_end(&mut self, col: usize) {}
}
