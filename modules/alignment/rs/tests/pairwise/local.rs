use biobit_alignment_rs::pairwise::{alignment, scoring, sw};

pub type Score = i32;
pub type Symbol = u8;

pub fn invrle(rle: &str) -> String {
    let gapfirst = alignment::Op::symbol(&alignment::Op::GapFirst);
    let gapsecond = alignment::Op::symbol(&alignment::Op::GapSecond);
    rle.chars()
        .map(|x| {
            if x == gapfirst {
                gapsecond
            } else if x == gapsecond {
                gapfirst
            } else {
                x
            }
        })
        .collect::<String>()
}

pub mod best {
    use super::*;

    type Engine = sw::Engine<
        Score,
        Symbol,
        sw::storage::Best<Score>,
        sw::traceback::TraceMatrix<Score>,
        scoring::Delegate<
            Score,
            Symbol,
            scoring::symbols::Equality<Score, u8>,
            scoring::gaps::Affine<Score>,
            scoring::equiv::Equality,
        >,
    >;

    struct Workload<'a> {
        seq1: (&'a [u8], usize),
        seq2: (&'a [u8], usize),
        score: Score,
        rle: &'a str,
    }

    fn ensure(aligner: &mut Engine, w: Workload<'_>) {
        let invrle = invrle(w.rle);

        for (seq1, seq2, rle) in [(w.seq1, w.seq2, w.rle), (w.seq2, w.seq1, &invrle)] {
            let mut result = aligner.scan_all(&seq1.0, &seq2.0);
            assert_eq!(result.len(), 1);
            let result = result.pop().unwrap();
            //     .expect(
            //     &*format!("Aligner failed: {:?} & {:?}", seq1.0, seq2.0)
            // );
            assert_eq!(result.seq1().start, seq1.1);
            assert_eq!(result.seq2().start, seq2.1);
            assert_eq!(*result.score(), w.score);
            assert_eq!(result.rle(), rle);
        }
    }

    fn test_empty(aligner: &mut Engine) {
        let workload: Vec<(&[u8], &[u8])> = vec![
            (b"ACGT", b""),
            (b"", b"ACGT"),
            (b"", b""),
            (b"ACGT", b"----"),
            (b"_", b"A"),
        ];

        for (seq1, seq2) in workload {
            let result = aligner.scan_all(&seq1, &seq2);
            assert!(result.is_empty());
        }
    }

    fn test_no_gaps(engine: &mut Engine) {
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

        for w in workload {
            ensure(engine, w);
        }
    }

    fn test_affine_gaps(engine: &mut Engine) {
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

        for w in workload {
            ensure(engine, w);
        }
    }

    fn test_free_gap_open(aligner: &mut Engine) {
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

        for w in workload {
            ensure(aligner, w);
        }
    }

    #[test]
    pub fn test_all() {
        let mut engine = Engine::new(
            sw::storage::Best::new(),
            sw::traceback::TraceMatrix::new(),
            scoring::compose(
                scoring::symbols::Equality::new(1, -2),
                scoring::gaps::Affine {
                    open: -5,
                    extend: -1,
                },
                scoring::equiv::Equality {},
            ),
        );
        test_empty(&mut engine);
        test_no_gaps(&mut engine);
        test_affine_gaps(&mut engine);

        engine.with_scoring(scoring::compose(
            scoring::symbols::Equality::new(1, -2),
            scoring::gaps::Affine {
                open: -1,
                extend: -1,
            },
            scoring::equiv::Equality {},
        ));
        test_free_gap_open(&mut engine);
    }
}

pub mod alloptimal {
    use std::iter::zip;

    use super::*;

    type Engine = sw::Engine<
        Score,
        Symbol,
        sw::storage::AllOptimal<Score>,
        sw::traceback::TraceMatrix<Score>,
        scoring::Delegate<
            Score,
            Symbol,
            scoring::symbols::Equality<Score, u8>,
            scoring::gaps::Affine<Score>,
            scoring::equiv::Equality,
        >,
    >;

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

    fn ensure(engine: &mut Engine, w: &mut Workload<'_>) {
        fn check<'a>(engine: &mut Engine, seq1: &'a [u8], seq2: &'a [u8], expected: &mut Vec<Hit>) {
            expected.sort_by_key(|x| (x.score, x.start));

            let mut hits = engine.scan_all(&seq1, &seq2);
            hits.sort_by_key(|x| (*x.score(), (x.seq1().start, x.seq2().start)));

            assert_eq!(hits.len(), expected.len());
            for (alignment, expected) in zip(&hits, expected) {
                assert_eq!(*alignment.score(), expected.score);
                assert_eq!(
                    (alignment.seq1().start, alignment.seq2().start),
                    expected.start
                );
                assert_eq!(alignment.rle(), expected.rle);
            }
        }

        check(engine, w.seq1, w.seq2, &mut w.hits);

        let invrle: Vec<_> = w.hits.iter().map(|x| invrle(x.rle)).collect();
        let mut invhits = w
            .hits
            .iter()
            .enumerate()
            .map(|(ind, x)| Hit {
                start: (x.start.1, x.start.0),
                score: x.score,
                rle: &invrle[ind],
            })
            .collect();
        check(engine, w.seq2, w.seq1, &mut invhits);
    }

    fn test_sequence_from_paper(engine: &mut Engine) {
        engine.with_scoring(scoring::compose(
            scoring::symbols::Equality::new(10, -9),
            scoring::gaps::Affine {
                open: -20,
                extend: -20,
            },
            scoring::equiv::Equality {},
        ));
        engine.storage().minscore = 21;
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
                Hit {
                    start: (0, 3),
                    score: 21,
                    rle: "2=1X1=",
                },
                Hit {
                    start: (2, 0),
                    score: 21,
                    rle: "1=1X2=",
                },
                Hit {
                    start: (13, 9),
                    score: 30,
                    rle: "3=",
                },
                Hit {
                    start: (21, 11),
                    score: 30,
                    rle: "3=",
                },
                Hit {
                    start: (21, 16),
                    score: 30,
                    rle: "3=",
                },
                Hit {
                    start: (11, 10),
                    score: 31,
                    rle: "2=1X2=",
                },
                Hit {
                    start: (19, 0),
                    score: 31,
                    rle: "3=1X1=",
                },
                Hit {
                    start: (0, 10),
                    score: 62,
                    rle: "1=1X1=1X6=",
                },
                Hit {
                    start: (8, 15),
                    score: 60,
                    rle: "6=",
                },
                Hit {
                    start: (5, 10),
                    score: 61,
                    rle: "5=1v2=1X2=",
                },
                Hit {
                    start: (8, 10),
                    score: 50,
                    rle: "5=",
                },
            ],
        };
        ensure(engine, &mut w);
    }

    #[test]
    fn test_all() {
        let mut engine = Engine::new(
            sw::storage::AllOptimal::new(0),
            sw::traceback::TraceMatrix::new(),
            scoring::compose(
                scoring::symbols::Equality::new(1, -2),
                scoring::gaps::Affine {
                    open: -5,
                    extend: -1,
                },
                scoring::equiv::Equality {},
            ),
        );
        test_sequence_from_paper(&mut engine);
    }
}
