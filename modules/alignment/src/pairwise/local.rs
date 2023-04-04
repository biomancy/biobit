use std::cmp::{max, min};
use std::ops::Range;

pub use super::{Alignable, Score, Symbol};
pub use super::alignment::{AlignmentOp, AlignmentOffset, AlignmentStep, CoalescedAlignmentStep, CoalescedAlignmentIter};
use super::alignment::utils;
pub use super::constraint::{ConstrainedPos, Constraint};
use super::scoring::ScoringScheme;

pub struct Alignment {
    pub score: Score,
    pub steps: Vec<AlignmentStep>,
    pub seq1: Range<usize>,
    pub seq2: Range<usize>,
}

impl Alignment {
    pub fn len(&self) -> usize {
        self.steps.iter().map(|x| x.len as usize).sum()
    }

    pub fn rle(&self) -> String {
        utils::rle(&self.steps, self.len())
    }

    pub fn prettify(&self, seq1: &str, seq2: &str) -> String {
        let seq1 = &seq1[self.seq1.start..self.seq1.end];
        let seq2 = &seq2[self.seq2.start..self.seq2.end];
        let total: usize = self.len();
        utils::prettify(seq1, seq2, &self.steps, total)
    }

    pub fn intersects(&self, other: &Alignment) -> bool {
        if max(self.seq1.start, other.seq1.start) >= min(self.seq1.end, other.seq1.end) {
            return false;
        }
        if max(self.seq2.start, other.seq2.start) >= min(self.seq2.end, other.seq2.end) {
            return false;
        }
        return utils::intersects(self.coalesced_steps(), other.coalesced_steps());
    }

    pub fn coalesced_steps(&self) -> impl Iterator<Item=CoalescedAlignmentStep> + '_ {
        CoalescedAlignmentIter {
            iter: self.steps.iter().peekable(),
            offset: AlignmentOffset { seq1: self.seq1.start, seq2: self.seq2.start },
        }
    }
}

pub trait Aligner<S1: Alignable, S2: Alignable, SF: ScoringScheme> {
    fn with_scoring(&mut self, scoring: SF);
    fn align(&mut self, seq1: &S1, seq2: &S2) -> Result<Alignment, ()>;
}

pub trait MultiAlignerConfig<SF: ScoringScheme> {
    fn set_scoring(&mut self, scoring: SF);
    fn set_min_score_thr(&mut self, thr: Score);
    // fn set_min_matches_run_thr(&mut self, thr: Score);
}

pub trait MultiAligner<S1: Alignable, S2: Alignable, SF: ScoringScheme> : MultiAlignerConfig<SF> {
    fn align(&mut self, seq1: &S1, seq2: &S2, saveto: &mut Vec<Alignment>);
    fn uptriangle(&mut self, seq1: &S1, seq2: &S2, offset: usize, saveto: &mut Vec<Alignment>);
}


#[cfg(test)]
pub mod test_suite {
    use super::*;
    use super::super::scoring;

    type TestAligner<'a> = dyn Aligner<
        &'a [u8], &'a [u8],
        scoring::Delegate<scoring::symbols::MatchMismatch, scoring::gaps::Affine>
    >;

    type TestMultiAligner<'a> = dyn MultiAligner<
        &'a [u8], &'a [u8],
        scoring::Delegate<scoring::symbols::MatchMismatch, scoring::gaps::Affine>
    >;

    pub fn invrle(rle: &str) -> String {
        let gapfirst = AlignmentOp::symbol(&AlignmentOp::GapFirst);
        let gapsecond = AlignmentOp::symbol(&AlignmentOp::GapSecond);
        rle.chars().map(|x|
            if x == gapfirst {
                gapsecond
            } else if x == gapsecond {
                gapfirst
            } else {
                x
            }
        ).collect::<String>()
    }


    pub mod best {
        use super::*;

        struct Workload<'a> {
            seq1: (&'a [u8], usize),
            seq2: (&'a [u8], usize),
            score: Score,
            rle: &'a str,
        }

        fn ensure<'a>(aligner: &mut TestAligner<'a>, w: Workload<'a>) {
            let invrle = invrle(w.rle);

            for (seq1, seq2, rle) in [
                (w.seq1, w.seq2, w.rle),
                (w.seq2, w.seq1, &invrle)
            ] {
                let result = aligner.align(&seq1.0, &seq2.0).expect(
                    &*format!("Aligner failed: {:?} & {:?}", seq1.0, seq2.0)
                );
                assert_eq!(result.seq1.start, seq1.1);
                assert_eq!(result.seq2.start, seq2.1);
                assert_eq!(result.score, w.score);
                assert_eq!(result.rle(), rle);
            }
        }

        fn test_empty(aligner: &mut TestAligner) {
            let workload: Vec<(&[u8], &[u8])> = vec![
                (b"ACGT", b""), (b"", b"ACGT"), (b"", b""),
                (b"ACGT", b"----"), (b"_", b"A"),
            ];

            for (seq1, seq2) in workload {
                let result = aligner.align(&seq1, &seq2);
                assert!(result.is_err());
            }
        }

        fn test_no_gaps(aligner: &mut TestAligner) {
            let workload = vec![
                Workload {
                    seq1: (b"AAGAA", 1),
                    seq2: (b"AGA", 0),
                    score: 3,
                    rle: "3=",
                },
                Workload {
                    seq1: (b"AGTCCCGTGTCCCAGGGG", 0),
                    seq2: (b"AGTC", 0),
                    score: 4,
                    rle: "4=",
                },
                Workload {
                    seq1: (b"CGCGCGCGTTT", 6),
                    seq2: (b"CGTTT", 0),
                    score: 5,
                    rle: "5=",
                },
                Workload {
                    seq1: (b"AAAGGGAGGGTTTA", 3),
                    seq2: (b"GGGGGGG", 0),
                    score: 4,
                    rle: "3=1X3=",
                },
                Workload {
                    seq1: (b"AAAA", 0),
                    seq2: (b"AAAA", 0),
                    score: 4,
                    rle: "4=",
                },
                Workload {
                    seq1: (b"NNNN==*===*===*==", 7),
                    seq2: (b"++++=============+++", 4),
                    score: 4,
                    rle: "3=1X3=",
                },
                Workload {
                    seq1: (b"NNNN===*===*===*===*===", 4),
                    seq2: (b"===================", 0),
                    score: 7,
                    rle: "3=1X3=1X3=1X3=1X3=",
                },
                Workload {
                    seq1: (b"AGAAAAAAAGGAAAAAAAGGGGG", 1),
                    seq2: (b"G", 0),
                    score: 1,
                    rle: "1=",
                },
            ];

            for mut w in workload {
                ensure(aligner, w);
            }
        }

        fn test_affine_gaps(aligner: &mut TestAligner) {
            let workload = vec![
                Workload {
                    seq1: (b"AAAAAAAAAAAAAAAA*********AAAAAAAAAAAAAAAA", 0),
                    seq2: (b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", 0),
                    score: 19,
                    rle: "16=9v16=",
                },
                Workload {
                    seq1: (b"ACGTACGTACGT****_________", 0),
                    seq2: (b"****ACGTACGTACGT_________ACGT*****", 4),
                    score: 13,
                    rle: "12=4v9=",
                },
            ];

            for mut w in workload {
                ensure(aligner, w);
            }
        }

        fn test_free_gap_open(aligner: &mut TestAligner) {
            let workload = vec![
                Workload {
                    seq1: (b"A***AAAAAAAA***AAAAAAAA***A", 4),
                    seq2: (b"AAAAAAAAAAAAAAAA", 0),
                    score: 13,
                    rle: "8=3v8=",
                },
                Workload {
                    seq1: (b"AAAAAAA**AAAAA*****", 0),
                    seq2: (b"___AAAAAAAAAAA", 3),
                    score: 9,
                    rle: "7=2v4=",
                },
            ];

            for mut w in workload {
                ensure(aligner, w);
            }
        }

        pub fn test_all(aligner: &mut TestAligner) {
            aligner.with_scoring(scoring::compose(
                scoring::symbols::MatchMismatch { same: 1, diff: -2 },
                scoring::gaps::Affine { open: -5, extend: -1 },
            ));
            test_empty(aligner);
            test_no_gaps(aligner);
            test_affine_gaps(aligner);

            aligner.with_scoring(scoring::compose(
                scoring::symbols::MatchMismatch { same: 1, diff: -2 },
                scoring::gaps::Affine { open: -1, extend: -1 },
            ));
            test_free_gap_open(aligner);
        }
    }

    pub mod alloptimal {
        use std::iter::zip;

        use super::*;

        #[derive(Clone)]
        struct Hit<'a> {
            start: (usize, usize),
            score: Score,
            rle: &'a str,
        }


        struct Workload<'a> {
            seq1: &'a [u8],
            seq2: &'a [u8],
            hits: Vec<Hit<'a>>,
        }

        fn ensure<'a>(aligner: &mut TestMultiAligner<'a>, w: &mut Workload<'a>) {
            fn check<'a>(aligner: &mut TestMultiAligner<'a>, seq1: &'a [u8], seq2: &'a [u8], expected: &mut Vec<Hit>) {
                expected.sort_by_key(|x| (x.score, x.start));

                let mut hits = Vec::with_capacity(expected.len());
                aligner.align(&seq1, &seq2, &mut hits);
                hits.sort_by_key(|x| (x.score, (x.seq1.start, x.seq2.start)));

                assert_eq!(hits.len(), expected.len());
                for (alignment, expected) in zip(&hits, expected) {
                    assert_eq!(alignment.score, expected.score);
                    assert_eq!((alignment.seq1.start, alignment.seq2.start), expected.start);
                    assert_eq!(alignment.rle(), expected.rle);
                }
            }

            check(aligner, w.seq1, w.seq2, &mut w.hits);


            let invrle: Vec<_> = w.hits.iter().map(|x| invrle(x.rle)).collect();
            let mut invhits = w.hits.iter().enumerate().map(|(ind, x)|
                Hit {
                    start: (x.start.1, x.start.0),
                    score: x.score,
                    rle: &invrle[ind],
                }
            ).collect();
            check(aligner, w.seq2, w.seq1, &mut invhits);
        }


        fn test_sequence_from_paper(aligner: &mut TestMultiAligner) {
            aligner.set_scoring(scoring::compose(
                scoring::symbols::MatchMismatch { same: 10, diff: -9 },
                scoring::gaps::Affine { open: -20, extend: -20 },
            ));
            aligner.set_min_score_thr(21);
            let mut w = Workload {
                seq1: b"CCAATCTACTACTGCTTGCAGTAC",
                seq2: b"AGTCCGAGGGCTACTCTACTGAAC",
                hits: vec![
                    // Hit { start: (17, 9), score: 20, rle: "2=" },
                    // Hit { start: (19, 6), score: 20, rle: "2=" },
                    // Hit { start: (5, 13), score: 20, rle: "2=" },
                    // Hit { start: (5, 18), score: 20, rle: "2=" },
                    // Hit { start: (2, 21), score: 20, rle: "2=" },
                    // Hit { start: (10, 22), score: 20, rle: "2=" },

                    Hit { start: (0, 3), score: 21, rle: "2=1X1=" },
                    Hit { start: (2, 0), score: 21, rle: "1=1X2=" },
                    Hit { start: (13, 9), score: 30, rle: "3=" },
                    Hit { start: (21, 11), score: 30, rle: "3=" },
                    Hit { start: (21, 16), score: 30, rle: "3=" },
                    Hit { start: (11, 10), score: 31, rle: "2=1X2=" },
                    Hit { start: (19, 0), score: 31, rle: "3=1X1=" },
                    Hit { start: (0, 10), score: 62, rle: "1=1X1=1X6=" },
                    Hit { start: (8, 15), score: 60, rle: "6=" },
                    Hit { start: (5, 10), score: 61, rle: "5=1v2=1X2=" },
                    Hit { start: (8, 10), score: 50, rle: "5=" },
                ],
            };
            ensure(aligner, &mut w);
            aligner.set_scoring(scoring::default());
        }


        pub fn test_all(aligner: &mut TestMultiAligner) {
            aligner.set_scoring(scoring::compose(
                scoring::symbols::MatchMismatch { same: 10, diff: -9 },
                scoring::gaps::Affine { open: -20, extend: -20 },
            ));
            test_sequence_from_paper(aligner);
        }
    }
}
