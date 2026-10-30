use crate::pairwise::scoring;
use crate::{Alignable, Score};

use super::Tracer;

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
//      scores(i) = C(i,j-1) before updating row i, C(i,j) afterward
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
pub struct FullScan<S: Score> {
    left: S,
    diagonal: S,
    scores: Vec<S>,
    gapcol: Vec<S>,
    gaprow: S,
}

impl<S: Score> FullScan<S> {
    pub fn new(capacity: usize) -> Self {
        Self {
            left: S::zero(),
            diagonal: S::zero(),
            scores: Vec::with_capacity(capacity),
            gapcol: Vec::with_capacity(capacity),
            gaprow: S::zero(),
        }
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.scores
            .reserve(capacity.saturating_sub(self.scores.len()));
        self.gapcol
            .reserve(capacity.saturating_sub(self.gapcol.len()));
    }
}

impl<S: Score> FullScan<S> {
    /// Solves the first `rows` cells of column 0 and initializes the rolling
    /// buffers for subsequent columns.
    /// Handling this boundary separately keeps the interior hot loop less variable.
    fn solve_first_col<Smb, Scheme, Seq1, Seq2, Trace>(
        &mut self,
        seq1: &Seq1,
        seq2: &Seq2,
        rows: usize,
        scorer: &mut Scheme,
        tracer: &mut Trace,
    ) where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
        Trace: Tracer<Score = S>,
    {
        tracer.first_col_start();
        let s2 = seq2.at(0);

        // No positive horizontal gap can exist in the first column.
        self.gapcol.clear();
        self.gapcol.resize(rows, S::zero());

        // Rebuild the first-column scores and reset the vertical gap at the top boundary.
        self.scores.clear();
        self.scores.reserve(rows);
        self.gaprow = S::zero();

        // At (0, 0), a positive alignment can only start with a symbol pair.
        let equiv = scorer.score(0, seq1.at(0), 0, s2);
        let score = if equiv > S::zero() {
            tracer.equivalent(0, 0, equiv);
            equiv
        } else {
            tracer.none(0, 0);
            S::zero()
        };
        self.scores.push(score);

        // Subsequent cells may extend an alignment through a vertical gap.
        for row in 1..rows {
            // Compute vertical gap candidates: open from the cell above,
            // or extend the existing vertical gap.
            let gap_open = scorer.seq1_gap_open(row);
            let gap_extend = scorer.seq1_gap_extend(row);
            debug_assert!(
                gap_open <= gap_extend && gap_extend <= S::zero(),
                "Gap scores must satisfy open <= extend <= 0 (seq1 position {row})"
            );
            let opened = self.scores[row - 1] + gap_open;
            let extended = self.gaprow + gap_extend;
            // Keep the best positive gap score, or reset to zero.
            self.gaprow = if opened > extended && opened > S::zero() {
                tracer.row_gap_open(row, 0, opened);
                opened
            } else if extended > S::zero() {
                tracer.row_gap_extend(row, 0, extended);
                extended
            } else {
                S::zero()
            };

            // Pick the best score at (row, 0): zero, a new symbol pair, or a vertical gap.
            let equiv = scorer.score(row, seq1.at(row), 0, s2);
            let score = if equiv > self.gaprow && equiv > S::zero() {
                tracer.equivalent(row, 0, equiv);
                equiv
            } else if self.gaprow > S::zero() {
                tracer.gap_row(row, 0, self.gaprow);
                self.gaprow
            } else {
                tracer.none(row, 0);
                S::zero()
            };

            self.scores.push(score);
        }

        tracer.first_col_end();
    }

    /// Updates cell (0, col) and prepares the rolling state for the rest of the column.
    #[inline(always)]
    fn solve_first_row<Smb, Scheme, Seq1, Seq2, Trace>(
        &mut self,
        seq1: &Seq1,
        seq2: &Seq2,
        col: usize,
        scorer: &mut Scheme,
        tracer: &mut Trace,
    ) where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
        Trace: Tracer<Score = S>,
    {
        // Reset the vertical gap at the top boundary. Save the previous
        // column's top score: it becomes the diagonal predecessor for row 1.
        self.gaprow = S::zero();
        self.diagonal = self.scores[0];

        // Compute horizontal gap candidates: open from the cell to the left,
        // or extend the existing horizontal gap.
        let gap_open = scorer.seq2_gap_open(col);
        let gap_extend = scorer.seq2_gap_extend(col);
        debug_assert!(
            gap_open <= gap_extend && gap_extend <= S::zero(),
            "Gap scores must satisfy open <= extend <= 0 (seq2 position {col})"
        );
        let opened = self.diagonal + gap_open;
        let extended = self.gapcol[0] + gap_extend;

        // Keep the best positive gap score, or reset to zero.
        self.gapcol[0] = if opened > extended && opened > S::zero() {
            tracer.col_gap_open(0, col, opened);
            opened
        } else if extended > S::zero() {
            tracer.col_gap_extend(0, col, extended);
            extended
        } else {
            S::zero()
        };

        // Pick the best score at (0, col): zero, a new symbol pair,
        // or a horizontal gap. The symbol pair starts a new alignment
        // because its diagonal predecessor lies on the zero boundary.
        let equiv = scorer.score(0, seq1.at(0), col, seq2.at(col));
        self.scores[0] = if equiv > self.gapcol[0] && equiv > S::zero() {
            tracer.equivalent(0, col, equiv);
            equiv
        } else if self.gapcol[0] > S::zero() {
            tracer.gap_col(0, col, self.gapcol[0]);
            self.gapcol[0]
        } else {
            tracer.none(0, col);
            S::zero()
        };
    }

    /// Visits cells satisfying `row + col + antidiagonal_offset < seq1.len()`, clipped to seq2.
    /// Increasing `antidiagonal_offset` moves the boundary toward (0, 0),
    /// reducing the scanned region. An offset at least `seq1.len()` visits no cells.
    ///
    /// For two sequences of length 6, rows index seq1 and columns index seq2:
    ///
    /// ```text
    /// antidiagonal_offset = 0     antidiagonal_offset = 2
    ///       0 1 2 3 4 5                0 1 2 3 4 5
    ///     0 x x x x x x              0 x x x x . .
    ///     1 x x x x x .              1 x x x . . .
    ///     2 x x x x . .              2 x x . . . .
    ///     3 x x x . . .              3 x . . . . .
    ///     4 x x . . . .              4 . . . . . .
    ///     5 x . . . . .              5 . . . . . .
    /// ```
    pub fn scan_up_triangle<Smb, Scheme, Seq1, Seq2, Trace>(
        &mut self,
        seq1: &Seq1,
        seq2: &Seq2,
        antidiagonal_offset: usize,
        scorer: &mut Scheme,
        tracer: &mut Trace,
    ) where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
        Trace: Tracer<Score = S>,
    {
        let rows = seq1.len().saturating_sub(antidiagonal_offset);
        let cols = seq2.len().min(rows);
        if cols == 0 {
            return;
        }

        self.solve_first_col(seq1, seq2, rows, scorer, tracer);
        for col in 1..cols {
            tracer.col_start(col);

            self.solve_first_row(seq1, seq2, col, scorer, tracer);

            for row in 1..(rows - col) {
                self.left = self.scores[row];

                // Vertical/row gaps
                let mut opened = self.scores[row - 1] + scorer.seq1_gap_open(row);
                let mut extended = self.gaprow + scorer.seq1_gap_extend(row);

                self.gaprow = if opened > extended && opened > S::zero() {
                    tracer.row_gap_open(row, col, opened);
                    opened
                } else if extended > S::zero() {
                    tracer.row_gap_extend(row, col, extended);
                    extended
                } else {
                    S::zero()
                };

                // Horizontal/column gaps
                opened = self.left + scorer.seq2_gap_open(col);
                extended = self.gapcol[row] + scorer.seq2_gap_extend(col);

                self.gapcol[row] = if opened > extended && opened > S::zero() {
                    tracer.col_gap_open(row, col, opened);
                    opened
                } else if extended > S::zero() {
                    tracer.col_gap_extend(row, col, extended);
                    extended
                } else {
                    S::zero()
                };

                // Best scores
                let equiv = self.diagonal + scorer.score(row, seq1.at(row), col, seq2.at(col));
                self.scores[row] =
                    if equiv > self.gaprow && equiv > self.gapcol[row] && equiv > S::zero() {
                        tracer.equivalent(row, col, equiv);
                        equiv
                    } else if self.gapcol[row] > self.gaprow && self.gapcol[row] > S::zero() {
                        tracer.gap_col(row, col, self.gapcol[row]);
                        self.gapcol[row]
                    } else if self.gaprow > S::zero() {
                        tracer.gap_row(row, col, self.gaprow);
                        self.gaprow
                    } else {
                        tracer.none(row, col);
                        S::zero()
                    };

                self.diagonal = self.left;
            }

            tracer.col_end(col);
        }
    }

    pub fn scan_all<Smb, Scheme, Seq1, Seq2, Trace>(
        &mut self,
        seq1: &Seq1,
        seq2: &Seq2,
        scorer: &mut Scheme,
        tracer: &mut Trace,
    ) where
        Scheme: scoring::Scheme<Symbol = Smb, Score = S>,
        Seq1: Alignable<Symbol = Smb>,
        Seq2: Alignable<Symbol = Smb>,
        Trace: Tracer<Score = S>,
    {
        if seq1.len() == 0 || seq2.len() == 0 {
            return;
        }

        self.solve_first_col(seq1, seq2, seq1.len(), scorer, tracer);
        for col in 1..seq2.len() {
            tracer.col_start(col);

            self.solve_first_row(seq1, seq2, col, scorer, tracer);

            for row in 1..seq1.len() {
                self.left = self.scores[row];

                // Vertical/row gaps
                let mut opened = self.scores[row - 1] + scorer.seq1_gap_open(row);
                let mut extended = self.gaprow + scorer.seq1_gap_extend(row);

                self.gaprow = if opened > extended && opened > S::zero() {
                    tracer.row_gap_open(row, col, opened);
                    opened
                } else if extended > S::zero() {
                    tracer.row_gap_extend(row, col, extended);
                    extended
                } else {
                    S::zero()
                };

                // Horizontal/column gaps
                opened = self.left + scorer.seq2_gap_open(col);
                extended = self.gapcol[row] + scorer.seq2_gap_extend(col);

                self.gapcol[row] = if opened > extended && opened > S::zero() {
                    tracer.col_gap_open(row, col, opened);
                    opened
                } else if extended > S::zero() {
                    tracer.col_gap_extend(row, col, extended);
                    extended
                } else {
                    S::zero()
                };

                // Best scores
                let equiv = self.diagonal + scorer.score(row, seq1.at(row), col, seq2.at(col));
                self.scores[row] =
                    if equiv > self.gaprow && equiv > self.gapcol[row] && equiv > S::zero() {
                        tracer.equivalent(row, col, equiv);
                        equiv
                    } else if self.gapcol[row] > self.gaprow && self.gapcol[row] > S::zero() {
                        tracer.gap_col(row, col, self.gapcol[row]);
                        self.gapcol[row]
                    } else if self.gaprow > S::zero() {
                        tracer.gap_row(row, col, self.gaprow);
                        self.gaprow
                    } else {
                        tracer.none(row, col);
                        S::zero()
                    };

                self.diagonal = self.left;
            }

            tracer.col_end(col);
        }
    }
}
