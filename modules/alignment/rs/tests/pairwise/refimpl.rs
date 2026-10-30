//! Simple reference aligner, scoring scheme, and checks for completed alignments.

use biobit_alignment_rs::pairwise::{Op, alignment::Alignment, scoring, sw};
use scoring::{gaps::Scorer as _, symbols::Scorer as _};

type Aln = Alignment<i32, u8, usize, usize>;
pub type Engine<Store> = sw::Engine<i32, u8, Store, sw::traceback::TraceMatrix<i32>, Scores>;

#[derive(Clone, Copy, Debug)]
pub struct Scores {
    pub equal: i32,
    pub different: i32,
    pub open: i32,
    pub extend: i32,
    pub positional: bool,
}

impl scoring::symbols::Scorer for Scores {
    type Score = i32;
    type Symbol = u8;

    fn score(&self, row: usize, a: &u8, col: usize, b: &u8) -> i32 {
        if a == b {
            self.equal + i32::from(self.positional && (row + col).is_multiple_of(2))
        } else {
            self.different
        }
    }
}

impl scoring::gaps::Scorer for Scores {
    type Score = i32;

    fn seq1_gap_open(&self, row: usize) -> i32 {
        self.open - i32::from(self.positional && row.is_multiple_of(2))
    }

    fn seq1_gap_extend(&self, row: usize) -> i32 {
        self.extend - i32::from(self.positional && !row.is_multiple_of(2))
    }

    fn seq2_gap_open(&self, col: usize) -> i32 {
        self.open - i32::from(self.positional && !col.is_multiple_of(2))
    }

    fn seq2_gap_extend(&self, col: usize) -> i32 {
        self.extend - i32::from(self.positional && col.is_multiple_of(2))
    }
}

impl scoring::equiv::Classifier for Scores {
    type Symbol = u8;

    fn classify(&self, a: &u8, b: &u8) -> scoring::equiv::Type {
        if a == b {
            scoring::equiv::Type::Match
        } else {
            scoring::equiv::Type::Mismatch
        }
    }
}

impl scoring::Scheme for Scores {
    type Score = i32;
    type Symbol = u8;
}

// Dummy implementation: enumerate gap lengths instead of maintaining affine-gap states.
// It does not use the production scanner, storage, or traceback implementation.
fn dummy_best_score(a: &[u8], b: &[u8], scores: Scores, antidiagonal_offset: Option<usize>) -> i32 {
    let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];
    let mut best = 0;
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let (row, col) = (i - 1, j - 1);
            if antidiagonal_offset.is_some_and(|antidiagonal_offset| {
                row + col >= a.len().saturating_sub(antidiagonal_offset)
            }) {
                continue;
            }
            let diagonal = matrix[i - 1][j - 1] + scores.score(row, &a[row], col, &b[col]);
            let vertical = (1..=i)
                .map(|len| {
                    matrix[i - len][j]
                        + scores.seq1_gap_open(i - len)
                        + (i - len + 1..i)
                            .map(|row| scores.seq1_gap_extend(row))
                            .sum::<i32>()
                })
                .max()
                .unwrap();
            let horizontal = (1..=j)
                .map(|len| {
                    matrix[i][j - len]
                        + scores.seq2_gap_open(j - len)
                        + (j - len + 1..j)
                            .map(|col| scores.seq2_gap_extend(col))
                            .sum::<i32>()
                })
                .max()
                .unwrap();
            matrix[i][j] = 0.max(diagonal).max(vertical).max(horizontal);
            best = best.max(matrix[i][j]);
        }
    }
    best
}

pub fn assert_valid_alignment(
    alignment: &Aln,
    a: &[u8],
    b: &[u8],
    scores: Scores,
    antidiagonal_offset: Option<usize>,
) {
    assert!(!alignment.is_empty());
    assert!(*alignment.score() > 0);
    let (mut x, mut y) = (alignment.seq1().start, alignment.seq2().start);
    let mut previous = None;
    let mut total = 0;
    for step in alignment.steps() {
        for _ in 0..*step.len() {
            let (row, col) = match step.op() {
                Op::Match | Op::Mismatch => {
                    assert_eq!(*step.op() == Op::Match, a[x] == b[y]);
                    total += scores.score(x, &a[x], y, &b[y]);
                    let cell = (x, y);
                    x += 1;
                    y += 1;
                    cell
                }
                Op::GapFirst => {
                    assert!(x < a.len());
                    total += if previous == Some(Op::GapFirst) {
                        scores.seq1_gap_extend(x)
                    } else {
                        scores.seq1_gap_open(x)
                    };
                    let cell = (x, y.checked_sub(1).expect("gap before any aligned symbols"));
                    x += 1;
                    cell
                }
                Op::GapSecond => {
                    assert!(y < b.len());
                    total += if previous == Some(Op::GapSecond) {
                        scores.seq2_gap_extend(y)
                    } else {
                        scores.seq2_gap_open(y)
                    };
                    let cell = (x.checked_sub(1).expect("gap before any aligned symbols"), y);
                    y += 1;
                    cell
                }
                Op::Equivalent => panic!("equality classification must disambiguate every symbol"),
            };
            if let Some(antidiagonal_offset) = antidiagonal_offset {
                assert!(row + col < a.len().saturating_sub(antidiagonal_offset));
            }
            previous = Some(*step.op());
        }
    }
    assert_eq!((x, y), (alignment.seq1().end, alignment.seq2().end));
    assert_eq!(total, *alignment.score(), "{alignment:?}, {scores:?}");
}

pub fn compare<Store>(
    engine: &mut Engine<Store>,
    a: &[u8],
    b: &[u8],
    scores: Scores,
    antidiagonal_offset: Option<usize>,
) where
    Store: sw::storage::Storage + sw::algo::Tracer<Score = i32>,
{
    let hits = match antidiagonal_offset {
        Some(antidiagonal_offset) => engine.scan_up_triangle(&a, &b, antidiagonal_offset),
        None => engine.scan_all(&a, &b),
    };
    assert_eq!(
        hits.iter().map(|hit| *hit.score()).max().unwrap_or(0),
        dummy_best_score(a, b, scores, antidiagonal_offset),
        "a={a:?}, b={b:?}, scores={scores:?}, antidiagonal_offset={antidiagonal_offset:?}, hits={hits:?}",
    );
    if hits.is_empty() {
        assert!(
            engine.storage().finalize().is_empty(),
            "an empty scan must discard previously stored results",
        );
    }
    for alignment in hits {
        assert_valid_alignment(&alignment, a, b, scores, antidiagonal_offset);
    }
}
