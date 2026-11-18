use crate::Score;
use crate::pairwise::dp::{
    Cursor,
    layout::Layout,
    tracer::{CellScores, Trace, Tracer},
};

/// Number of rows in a full tracer callback. The last batch in a column may be shorter.
/// Discriminants are base-two exponents. Use `rows()` to get the row count.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum ChunkSize {
    Rows1 = 0,
    Rows4 = 2,
    Rows16 = 4,
    Rows64 = 6,
    Rows128 = 7,
    Rows256 = 8,
    Rows512 = 9,
    #[default]
    Rows1024 = 10,
}

impl ChunkSize {
    /// Supported sizes in increasing order.
    pub const ALL: [Self; 8] = [
        Self::Rows1,
        Self::Rows4,
        Self::Rows16,
        Self::Rows64,
        Self::Rows128,
        Self::Rows256,
        Self::Rows512,
        Self::Rows1024,
    ];

    /// Returns the row count represented by this exponent.
    pub const fn rows(self) -> usize {
        1usize << (self as u8)
    }
}

pub(super) struct Chunk<S, const N: usize> {
    scores: [CellScores<S>; N],
    traces: [Trace; N],
}

impl<S: Score, const N: usize> Chunk<S, N> {
    pub fn new() -> Self {
        let zero = S::zero();
        Self {
            scores: [CellScores {
                best: zero,
                row_gap: zero,
                col_gap: zero,
            }; N],
            traces: [Trace::Start; N],
        }
    }

    /// Scans a nonempty column whose first cell has already been solved.
    #[inline(always)]
    pub fn scan(
        &mut self,
        pointer: &mut Cursor,
        total_rows: usize,
        layout: &impl Layout,
        tracer: &mut impl Tracer<Score = S>,
        first: (CellScores<S>, Trace),
        mut solve: impl FnMut(usize) -> (CellScores<S>, Trace),
    ) {
        debug_assert_eq!(pointer.row, 0);
        (self.scores[0], self.traces[0]) = first;

        // Solve full rows first, then any remaining rows.
        let full_rows = total_rows / N * N;
        if full_rows != 0 {
            for (i, (score, trace)) in self.scores[1..]
                .iter_mut()
                .zip(&mut self.traces[1..])
                .enumerate()
            {
                (*score, *trace) = solve(i + 1);
            }
            loop {
                tracer.chunk(layout, pointer, &self.scores, &self.traces);
                pointer.next_rows(N);
                if pointer.row == full_rows {
                    break;
                }
                for (i, (score, trace)) in self.scores.iter_mut().zip(&mut self.traces).enumerate()
                {
                    (*score, *trace) = solve(pointer.row + i);
                }
            }
        }

        let remaining = total_rows - pointer.row;
        if remaining != 0 {
            let scores = &mut self.scores[..remaining];
            let traces = &mut self.traces[..remaining];
            // A column shorter than one full chunk already has row zero in its tail.
            let start = usize::from(full_rows == 0);
            let row = pointer.row + start;
            for (i, (score, trace)) in scores[start..]
                .iter_mut()
                .zip(&mut traces[start..])
                .enumerate()
            {
                (*score, *trace) = solve(row + i);
            }
            tracer.tail(layout, pointer, scores, traces);
            pointer.next_rows(remaining);
        }
    }
}
