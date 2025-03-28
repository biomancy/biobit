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
// I(i,j) = {
//      I(i,j-1) + gap-extend
//      C(i,j-1) + gap-open
// }

// Optimization:
// C:
//      scores(i) = current C column
//      diagonal = C(i-1,j-1)
//      left = C(i-1, j)
// I:
//      gapcol(i) = current I column (horizontal seq2 gaps)
// D:
//      gaprow = current D(i-1) value (vertical seq1 gaps)

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
        debug_assert!(self.scores.capacity() == self.gapcol.capacity());
        if self.scores.capacity() < capacity {
            let diff = self.scores.capacity() - capacity;
            self.scores.reserve(diff);
            self.gapcol.reserve(diff);
        }
    }
}

impl<S: Score> FullScan<S> {
    fn solve_first_col<Smb, Scheme, Seq1, Seq2, Trace>(
        &mut self,
        seq1: &Seq1,
        seq2: &Seq2,
        skip_last: usize,
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
        // Initialize gap-col
        // (required in later recursion, set to 0 for faster initialization)
        self.gapcol.clear();
        self.gapcol.resize(seq1.len(), S::zero());

        // Initialize scores, the first column - only equiv are allowed
        // (assuming that gap-row scores will be always <= 0)
        self.scores.clear();
        // TODO: verify that extend is faster
        self.scores.extend((0..seq1.len() - skip_last).map(|row| {
            let equiv = scorer.score(row, seq1.at(row), 0, s2);
            if equiv > S::zero() {
                tracer.equivalent(row, 0, equiv);
                equiv
            } else {
                tracer.none(row, 0);
                S::zero()
            }
        }));

        tracer.first_col_end();
    }

    pub fn scan_up_triangle<Smb, Scheme, Seq1, Seq2, Trace>(
        &mut self,
        seq1: &Seq1,
        seq2: &Seq2,
        offset: usize,
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

        self.solve_first_col(seq1, seq2, offset, scorer, tracer);
        for col in 1..(seq2.len() - offset) {
            tracer.col_start(col);

            self.gaprow = S::zero();
            self.diagonal = self.scores[0];

            // First element
            // (assuming that gap-col scores will be always <= 0)
            let equiv = scorer.score(0, seq1.at(0), col, seq2.at(col));
            self.scores[0] = if equiv > S::zero() {
                tracer.equivalent(0, col, equiv);
                equiv
            } else {
                tracer.none(0, col);
                S::zero()
            };

            for row in 1..(seq1.len() - col - offset) {
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

        self.solve_first_col(seq1, seq2, 0, scorer, tracer);
        for col in 1..seq2.len() {
            tracer.col_start(col);

            self.gaprow = S::zero();
            self.diagonal = self.scores[0];

            // First element
            // (assuming that gap-col scores will be always <= 0)
            let equiv = scorer.score(0, seq1.at(0), col, seq2.at(col));
            self.scores[0] = if equiv > S::zero() {
                tracer.equivalent(0, col, equiv);
                equiv
            } else {
                tracer.none(0, col);
                S::zero()
            };

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
