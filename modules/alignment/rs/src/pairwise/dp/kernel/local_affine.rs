use crate::pairwise::scoring;
use crate::{Alignable, Score};

use super::super::tracer::{CellScores, Trace, Tracer};
use super::super::{Cursor, Scan, layout::Layout};
use super::chunk::{Chunk, ChunkSize};

// Optimal alignments in linear space: https://doi.org/10.1093/bioinformatics/4.1.11

// For the alignment of A and B of length i and j:
// C(i, j) = max score
// D(i, j) = max score where A(i) is deleted
// I(i, j) = max score where B(i) is deleted

// Dynamic programming equations:
// C(i,j) = max {
//      D(i,j)
//      I(i,j)
//      C(i-1,j-1) + equiv(ai, bj)
//      0
// }
// D(i,j) = max {
//      D(i-1,j) + gap-extend
//      C(i-1,j) + gap-open
// }
// I(i,j) = max {
//      I(i,j-1) + gap-extend
//      C(i,j-1) + gap-open
// }

// Optimization:
// C:
//      best(i) = C(i,j-1) before updating row i, C(i,j) afterward
//      diagonal = C(i-1,j-1)
//      left = C(i,j-1)
// I:
//      gapcol(i) = I(i,j-1) before updating row i, I(i,j) afterward
// D:
//      gaprow = D(i-1,j) before updating row i, D(i,j) afterward

/// Local affine-gap alignment with `gap_open <= gap_extend <= 0` at each gap position.
/// Opening must not be cheaper than extending, since gaps open from the best
/// score, which may itself correspond to a gap one position earlier.
///
/// Individual scores and all intermediate sums must fit in `S`;
/// arithmetic overflow is not explicitly checked.
pub struct LocalAffine<S: Score> {
    rolling: RollingScores<S>,
    chunk_size: ChunkSize,
}

struct RollingScores<S> {
    best: Vec<S>,
    gapcol: Vec<S>,
}

struct ColumnState<S> {
    above: S,
    diagonal: S,
    gap_row: S,
}

impl<S: Score> LocalAffine<S> {
    pub fn new(capacity: usize) -> Self {
        Self {
            rolling: RollingScores {
                best: Vec::with_capacity(capacity),
                gapcol: Vec::with_capacity(capacity),
            },
            chunk_size: ChunkSize::default(),
        }
    }

    /// Sets the size of a row batch. Complete batches are delivered as fixed-size
    /// chunks. The last, incomplete batch of each column is delivered as a tail.
    pub fn set_chunk_size(&mut self, size: ChunkSize) {
        self.chunk_size = size;
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.rolling
            .best
            .reserve(capacity.saturating_sub(self.rolling.best.len()));
        self.rolling
            .gapcol
            .reserve(capacity.saturating_sub(self.rolling.gapcol.len()));
    }
}

impl<S: Score> LocalAffine<S> {
    #[inline(always)]
    fn choose_gap(opened: S, extended: S) -> (S, bool) {
        let zero = S::zero();
        let extends = (extended >= opened) & (extended > zero);
        (opened.max(extended).max(zero), extends)
    }

    #[inline(always)]
    fn choose_cell(
        equiv: S,
        (row_gap, row_extends): (S, bool),
        (col_gap, col_extends): (S, bool),
    ) -> (CellScores<S>, Trace) {
        // Gap scores have already been clamped to zero. Gaps win ties against
        // the diagonal, and the row gap wins ties between the two gaps.
        let gap = row_gap.max(col_gap);
        let equiv_wins = equiv > gap;
        let col_wins = col_gap > row_gap;
        let row_wins = !col_wins & (row_gap > S::zero());

        // A row gap sets the low bit, a column gap sets the high bit, and a
        // winning diagonal sets both bits regardless of the gap scores.
        let direction = u8::from(equiv_wins | row_wins) | (u8::from(equiv_wins | col_wins) << 1);
        (
            CellScores {
                best: equiv.max(gap),
                row_gap,
                col_gap,
            },
            // SAFETY: The direction combines two boolean values in bits 0 and 1.
            unsafe { Trace::from_direction_bits(direction, row_extends, col_extends) },
        )
    }

    /// Solves the first visited column and initializes the rolling buffers.
    /// Handling this boundary separately keeps the interior hot loop less variable.
    fn solve_first_col<const N: usize, Smb, Scheme, Seq1, Seq2, T>(
        rolling: &mut RollingScores<S>,
        chunk: &mut Chunk<S, N>,
        (seq1, seq2): (&Seq1, &Seq2),
        (pointer, rows): (&mut Cursor, usize),
        layout: &impl Layout,
        scorer: &mut Scheme,
        tracer: &mut T,
    ) where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
        T: Tracer<Score = S>,
    {
        let col = pointer.col;
        let s2 = seq2.at(col);

        let equiv = scorer.score(0, seq1.at(0), col, s2);
        let first = Self::choose_cell(equiv, (S::zero(), false), (S::zero(), false));
        rolling.best[0] = first.0.best;

        // Track the vertical gap at the top boundary.
        let mut gap_row = S::zero();

        chunk.scan(
            pointer,
            rows,
            layout,
            tracer,
            first,
            #[inline(always)]
            |row| {
                // Compute vertical gap candidates: open from the cell above,
                // or extend the existing vertical gap.
                let gap_open = scorer.seq1_gap_open(row);
                let gap_extend = scorer.seq1_gap_extend(row);
                debug_assert!(
                    gap_open <= gap_extend && gap_extend <= S::zero(),
                    "Gap scores must satisfy open <= extend <= 0 (seq1 position {row})"
                );

                let opened = rolling.best[row - 1] + gap_open;
                let extended = gap_row + gap_extend;
                let row_gap = Self::choose_gap(opened, extended);
                gap_row = row_gap.0;

                let equiv = scorer.score(row, seq1.at(row), col, s2);
                let cell = Self::choose_cell(equiv, row_gap, (S::zero(), false));
                rolling.best[row] = cell.0.best;
                cell
            },
        );
    }

    /// Updates row zero and prepares the rolling state for row one.
    #[inline(always)]
    fn solve_first_row<Smb, Scheme, Seq1, Seq2>(
        rolling: &mut RollingScores<S>,
        seq1: &Seq1,
        seq2: &Seq2,
        col: usize,
        scorer: &mut Scheme,
    ) -> (ColumnState<S>, (CellScores<S>, Trace))
    where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
    {
        let diagonal = rolling.best[0];
        let gap_open = scorer.seq2_gap_open(col);
        let gap_extend = scorer.seq2_gap_extend(col);
        debug_assert!(
            gap_open <= gap_extend && gap_extend <= S::zero(),
            "Gap scores must satisfy open <= extend <= 0 (seq2 position {col})"
        );

        let col_gap = Self::choose_gap(diagonal + gap_open, rolling.gapcol[0] + gap_extend);
        rolling.gapcol[0] = col_gap.0;

        let equiv = scorer.score(0, seq1.at(0), col, seq2.at(col));
        let first = Self::choose_cell(equiv, (S::zero(), false), col_gap);

        rolling.best[0] = first.0.best;
        (
            ColumnState {
                above: first.0.best,
                diagonal,
                gap_row: S::zero(),
            },
            first,
        )
    }

    #[inline(always)]
    fn solve_interior_cell<Smb, Scheme, Seq1, Seq2>(
        rolling: &mut RollingScores<S>,
        (row, col): (usize, usize),
        state: &mut ColumnState<S>,
        seq1: &Seq1,
        seq2: &Seq2,
        scorer: &mut Scheme,
    ) -> (CellScores<S>, Trace)
    where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
    {
        let left = rolling.best[row];
        let row_gap = Self::choose_gap(
            state.above + scorer.seq1_gap_open(row),
            state.gap_row + scorer.seq1_gap_extend(row),
        );
        let col_gap = Self::choose_gap(
            left + scorer.seq2_gap_open(col),
            rolling.gapcol[row] + scorer.seq2_gap_extend(col),
        );

        let equiv = state.diagonal + scorer.score(row, seq1.at(row), col, seq2.at(col));
        let (scores, trace) = Self::choose_cell(equiv, row_gap, col_gap);

        rolling.best[row] = scores.best;
        rolling.gapcol[row] = scores.col_gap;

        *state = ColumnState {
            above: scores.best,
            diagonal: left,
            gap_row: scores.row_gap,
        };
        (scores, trace)
    }

    #[inline(always)]
    fn solve_interior_column<const N: usize, Smb, Scheme, Seq1, Seq2, T>(
        rolling: &mut RollingScores<S>,
        chunk: &mut Chunk<S, N>,
        (pointer, rows): (&mut Cursor, usize),
        layout: &impl Layout,
        (seq1, seq2): (&Seq1, &Seq2),
        scorer: &mut Scheme,
        tracer: &mut T,
    ) where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
        T: Tracer<Score = S>,
    {
        let col = pointer.col;
        let (mut state, first) = Self::solve_first_row(rolling, seq1, seq2, col, scorer);
        chunk.scan(
            pointer,
            rows,
            layout,
            tracer,
            first,
            #[inline(always)]
            |row| Self::solve_interior_cell(rolling, (row, col), &mut state, seq1, seq2, scorer),
        );
    }

    /// Scores the cells in the scan and reports them to the tracer in column-major order.
    /// Use [`Scan::layout`] on the same scan for subsequent traceback queries.
    pub fn scan<Seq1, Seq2>(
        &mut self,
        scan: &Scan<Seq1, Seq2, impl Layout>,
        scorer: &mut impl scoring::Scheme<Score = S, Symbol = Seq1::Symbol>,
        tracer: &mut impl Tracer<Score = S>,
    ) where
        Seq1: Alignable,
        Seq2: Alignable<Symbol = Seq1::Symbol>,
    {
        match self.chunk_size {
            ChunkSize::Rows1 => self.scan_with_chunk::<1, _, _>(scan, scorer, tracer),
            ChunkSize::Rows4 => self.scan_with_chunk::<4, _, _>(scan, scorer, tracer),
            ChunkSize::Rows16 => self.scan_with_chunk::<16, _, _>(scan, scorer, tracer),
            ChunkSize::Rows64 => self.scan_with_chunk::<64, _, _>(scan, scorer, tracer),
            ChunkSize::Rows128 => self.scan_with_chunk::<128, _, _>(scan, scorer, tracer),
            ChunkSize::Rows256 => self.scan_with_chunk::<256, _, _>(scan, scorer, tracer),
            ChunkSize::Rows512 => self.scan_with_chunk::<512, _, _>(scan, scorer, tracer),
            ChunkSize::Rows1024 => self.scan_with_chunk::<1024, _, _>(scan, scorer, tracer),
        }
    }

    fn scan_with_chunk<const N: usize, Seq1, Seq2>(
        &mut self,
        scan: &Scan<Seq1, Seq2, impl Layout>,
        scorer: &mut impl scoring::Scheme<Score = S, Symbol = Seq1::Symbol>,
        tracer: &mut impl Tracer<Score = S>,
    ) where
        Seq1: Alignable,
        Seq2: Alignable<Symbol = Seq1::Symbol>,
    {
        let (seq1, seq2) = scan.sequences();
        let layout = scan.layout();
        tracer.initialize(layout);

        let mut cols = layout.columns();
        let Some(first_col) = cols.next() else {
            return;
        };
        let rows = layout.rows(first_col);
        debug_assert_eq!(rows.start, 0);

        // Each row is visited in one contiguous interval. Its first visit sees
        // zero left-neighbor scores, and its scores can be ignored once it leaves.
        // See the layout documentation for more details.
        let max_rows = layout.max_rows();
        self.rolling.best.clear();
        self.rolling.best.resize(max_rows, S::zero());
        self.rolling.gapcol.clear();
        self.rolling.gapcol.resize(max_rows, S::zero());

        let mut chunk = Chunk::<S, N>::new();
        let mut pointer = Cursor::new(first_col);

        tracer.col_start(layout, pointer.col);
        Self::solve_first_col(
            &mut self.rolling,
            &mut chunk,
            (seq1, seq2),
            (&mut pointer, rows.len()),
            layout,
            scorer,
            tracer,
        );
        tracer.col_end(layout, pointer.col);

        for _ in cols {
            pointer.next_col();
            let rows = layout.rows(pointer.col);
            debug_assert_eq!(rows.start, 0);

            tracer.col_start(layout, pointer.col);
            Self::solve_interior_column(
                &mut self.rolling,
                &mut chunk,
                (&mut pointer, rows.len()),
                layout,
                (seq1, seq2),
                scorer,
                tracer,
            );
            tracer.col_end(layout, pointer.col);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CellScores, LocalAffine};
    use crate::pairwise::dp::tracer::BestTrace;

    #[test]
    fn gap_selection_extends_positive_ties_and_clears_inactive_gaps() {
        for (opened, extended, expected) in [
            (3, 2, (3, false)),
            (2, 3, (3, true)),
            (3, 3, (3, true)),
            (0, 0, (0, false)),
            (-1, 0, (0, false)),
            (0, -1, (0, false)),
            (i32::MIN, -1, (0, false)),
            (i32::MAX, i32::MAX, (i32::MAX, true)),
        ] {
            assert_eq!(LocalAffine::<i32>::choose_gap(opened, extended), expected);
        }
    }

    #[test]
    fn finish_cell_preserves_scores_and_trace_on_ties() {
        for (diagonal, row, col, (best, direction)) in [
            (i32::MIN, 0, 0, (0, BestTrace::Start)),
            (0, 0, 0, (0, BestTrace::Start)),
            (3, 2, 1, (3, BestTrace::Equivalent)),
            (3, 1, 2, (3, BestTrace::Equivalent)),
            (2, 3, 3, (3, BestTrace::GapRow)),
            (3, 3, 3, (3, BestTrace::GapRow)),
            (3, 1, 3, (3, BestTrace::GapCol)),
            (i32::MAX, i32::MAX, 0, (i32::MAX, BestTrace::GapRow)),
        ] {
            for (row_extends, col_extends) in
                [(false, false), (true, false), (false, true), (true, true)]
            {
                let (scores, trace) = LocalAffine::<i32>::choose_cell(
                    diagonal,
                    (row, row_extends),
                    (col, col_extends),
                );
                assert_eq!(
                    scores,
                    CellScores {
                        best,
                        row_gap: row,
                        col_gap: col,
                    }
                );
                assert_eq!(trace.best(), direction);
                assert_eq!(trace.row_gap_extends(), row_extends);
                assert_eq!(trace.col_gap_extends(), col_extends);
            }
        }
    }
}
