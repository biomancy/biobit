//! Correctness benchmarks against a simple reference aligner.
//!
//! The reference stores the full score matrix and tries every gap length.
//! We compare final best scores and independently validate the returned alignments.

use super::refimpl;
use biobit_alignment_rs::pairwise::sw;
use itertools::Itertools;

#[test]
fn full_and_triangle_scans_match_dummy_reference_and_return_valid_alignments() {
    let sequences: Vec<Vec<u8>> = (0..=4)
        .flat_map(|len| std::iter::repeat_n(*b"AC", len).multi_cartesian_product())
        .collect();
    for (equal, different, open, extend, positional) in [
        (3, -2, -1, -1, false),
        (2, -3, -4, -1, false),
        (3, -2, -1, 0, false),
        (1, 0, 0, 0, false),
        (4, -3, -3, -1, true),
    ] {
        let scores = refimpl::Scores {
            equal,
            different,
            open,
            extend,
            positional,
        };
        let mut best = refimpl::Engine::new(
            sw::storage::Best::new(),
            sw::traceback::TraceMatrix::new(),
            scores,
        );
        let mut all = refimpl::Engine::new(
            sw::storage::AllOptimal::new(0),
            sw::traceback::TraceMatrix::new(),
            scores,
        );
        for (a, b) in sequences.iter().cartesian_product(&sequences) {
            refimpl::compare(&mut best, a, b, scores, None);
            refimpl::compare(&mut all, a, b, scores, None);
            for antidiagonal_offset in (0..=a.len() + 1).chain([usize::MAX]) {
                refimpl::compare(&mut best, a, b, scores, Some(antidiagonal_offset));
                refimpl::compare(&mut all, a, b, scores, Some(antidiagonal_offset));
            }
        }
    }
}

#[test]
fn boundary_gaps_and_runs_longer_than_a_trace_step_preserve_score_and_coordinates() {
    let scores = refimpl::Scores {
        equal: 3,
        different: -2,
        open: -1,
        extend: 0,
        positional: false,
    };
    let mut engine = refimpl::Engine::new(
        sw::storage::Best::new(),
        sw::traceback::TraceMatrix::new(),
        scores,
    );
    for (a, b, expected_score) in [
        // A-C / AGC requires a gap in the first DP row: two matches and one
        // gap opening score 2 * 3 - 1 = 5. Swapping sequences tests the first column.
        (b"AC".to_vec(), b"AGC".to_vec(), 5),
        // A 270-base matching run exceeds the u8 step limit of 255 and must be
        // split during traceback without losing bases. Score: 270 * 3 = 810.
        (vec![b'A'; 270], vec![b'A'; 270], 810),
        // Two 8-base matching blocks separated by a 270-base gap. Splitting the
        // gap into u8 steps must preserve one opening charge, with free extension.
        // Score: (8 + 8) * 3 - 1 = 47.
        (
            [vec![b'A'; 8], vec![b'G'; 270], vec![b'C'; 8]].concat(),
            [vec![b'A'; 8], vec![b'C'; 8]].concat(),
            47,
        ),
    ] {
        for (a, b) in [(&a, &b), (&b, &a)] {
            let hits = engine.scan_all(a, b);
            assert_eq!(hits.len(), 1);
            let hit = &hits[0];
            assert_eq!(*hit.score(), expected_score);
            assert_eq!(hit.seq1(), &(0..a.len()));
            assert_eq!(hit.seq2(), &(0..b.len()));
            refimpl::assert_valid_alignment(hit, a, b, scores, None);
        }
    }
}

#[test]
fn all_optimal_retains_distinct_starts_and_resets_between_scans() {
    let scores = refimpl::Scores {
        equal: 1,
        different: -2,
        open: -5,
        extend: -1,
        positional: false,
    };
    let mut engine = refimpl::Engine::new(
        sw::storage::AllOptimal::new(0),
        sw::traceback::TraceMatrix::new(),
        scores,
    );
    // Every A starts a distinct one-base hit in each column. Reuse the engine
    // across smaller, empty, and larger scans to check that its index is reset.
    for (a, len) in [(b"ANA".as_slice(), 4096), (b"A", 2), (b"", 0), (b"ANA", 3)] {
        let b = vec![b'A'; len];
        let hits = engine.scan_all(&a, &b);
        let expected: Vec<_> = a
            .iter()
            .enumerate()
            .filter(|(_, base)| **base == b'A')
            .flat_map(|(row, _)| (0..len).map(move |col| (row..row + 1, col..col + 1)))
            .collect();
        let actual: Vec<_> = hits
            .iter()
            .map(|hit| (hit.seq1().clone(), hit.seq2().clone()))
            .collect();
        assert_eq!(actual, expected);
        for hit in hits {
            refimpl::assert_valid_alignment(&hit, a, &b, scores, None);
        }
    }
}
