use eyre::{Result, bail, ensure};
use rstar::{AABB, RTree, RTreeObject};

use biobit_alignment_rs::alignable::Reversed;
use biobit_alignment_rs::pairwise::alignment::Alignment;
use biobit_alignment_rs::pairwise::{alignment, scoring, sw};
use biobit_collections_rs::interval_tree::{BitsBuilder, Builder};
use biobit_core_rs::loc::{Interval, IntervalOp};

use crate::predict::filtering::Filter;
use crate::predict::scoring::Scoring;
use crate::predict::storage::AllOptimal;
use crate::repeats::{InvRepeat, InvSegment};

use super::storage::filtering::{EquivRunStats, SoftFilter};

struct Wrapper<S: scoring::Score>(Alignment<S, u8, usize, usize>, Vec<InvSegment<usize>>);

impl<S: scoring::Score> RTreeObject for Wrapper<S> {
    type Envelope = AABB<(isize, isize)>;

    fn envelope(&self) -> Self::Envelope {
        let seq1 = self.0.seq1();
        let seq2 = self.0.seq2();

        AABB::from_corners(
            (seq1.start as isize, seq2.start as isize),
            (seq1.end as isize, seq2.end as isize),
        )
    }
}

fn suppress_overlaps<S: scoring::Score>(mut candidates: Vec<Wrapper<S>>) -> Vec<Wrapper<S>> {
    // Sort from highest score to lowest by comparing b's score with a's.
    // Break ties by ascending coordinates.
    candidates.sort_unstable_by(|a, b| {
        b.0.score().cmp(a.0.score()).then_with(|| {
            (
                a.0.seq1().start,
                a.0.seq2().start,
                a.0.seq1().end,
                a.0.seq2().end,
            )
                .cmp(&(
                    b.0.seq1().start,
                    b.0.seq2().start,
                    b.0.seq1().end,
                    b.0.seq2().end,
                ))
        })
    });

    let mut passed: RTree<Wrapper<S>> = RTree::new();
    for candidate in candidates {
        if !passed
            .locate_in_envelope_intersecting(candidate.envelope())
            .any(|existing| existing.0.intersects(&candidate.0))
        {
            passed.insert(candidate);
        }
    }
    passed.drain().collect()
}

/// Predict inverted repeats from a DNA or RNA sequence.
/// Accepts A, C, G, U, T, and N in either case and converts T to U.
/// Returns an error with the byte index if any other byte is present.
/// Gap scores must satisfy `gap_open <= gap_extend <= 0`; otherwise returns an error.
pub fn run<S: scoring::Score>(
    seq: &[u8],
    filter: Filter<S>,
    scoring: Scoring<S>,
) -> Result<(Vec<InvRepeat<usize>>, Vec<S>)> {
    ensure!(
        scoring.gap_open <= scoring.gap_extend && scoring.gap_extend <= S::zero(),
        "Gap scores must satisfy gap_open <= gap_extend <= 0 (got gap_open={:?}, gap_extend={:?})",
        scoring.gap_open,
        scoring.gap_extend,
    );

    let mut normalized = Vec::with_capacity(seq.len());
    for (index, &base) in seq.iter().enumerate() {
        normalized.push(match base.to_ascii_uppercase() {
            b'T' => b'U',
            base @ (b'A' | b'C' | b'G' | b'U' | b'N') => base,
            _ => bail!(
                "Invalid nucleotide byte 0x{base:02X} at index {index}: expected A, C, G, U, T, or N"
            ),
        });
    }
    let seq = normalized.as_slice();
    let scoring = scoring::compose(
        scoring::symbols::RNAComplementarity::new(scoring.complementary, scoring.mismatch),
        scoring::gaps::Affine {
            open: scoring.gap_open,
            extend: scoring.gap_extend,
        },
        scoring::equiv::RNAComplementarity {},
    );
    let rois = filter.rois().to_vec();

    let mut aligner = sw::Engine::new(
        AllOptimal::new(filter, rois.clone()),
        sw::traceback::TraceMatrix::new(),
        scoring,
    );

    // sequence orientation:
    // ------>
    // ^    /
    // |   /
    // |  /
    // | /
    // |/
    // let seq = seq[..seq.len() - 1];
    let alignments = aligner.scan_up_triangle(&Reversed::new(&seq), &seq, 1);

    // During the alignment all overlaps between ROIs and alignments are judged based on diagonal
    // runs that can be either matches or mismatches. However, the threshold for ROI overlaps
    // assumed that the overlap calculated over matches only. Therefore, we need to recalculate
    // the overlap and filter out the alignments that do not meet the criteria.
    let (rois, filter) = {
        let (_, _, tracer) = aligner.dissolve();
        let (allopt, _, _) = tracer.dissolve();
        let filter = allopt.dissolve().0;

        let mut index = BitsBuilder::default();
        for roi in filter.rois() {
            index = index.add(*roi, ());
        }
        (index.build(), filter)
    };

    // Alignments are not guaranteed to be non-overlapping so we need to employ additional filtering
    // to suppress suboptimal overlapping alignments.
    let mut candidates = Vec::new();
    for aln in alignments.into_iter() {
        let mut intervals: Vec<InvSegment<usize>> = Vec::with_capacity(aln.steps().len());

        for step in aln.tracked_steps() {
            match step.step.op() {
                alignment::Op::Equivalent => {
                    let left =
                        Interval::new(step.start.seq2, step.start.seq2 + *step.step.len() as usize)
                            .unwrap();
                    let right = Interval::new(
                        seq.len() - step.start.seq1 - (*step.step().len() as usize),
                        seq.len() - step.start.seq1,
                    )
                    .unwrap();

                    // Traceback splits runs at u8::MAX. Rejoin contiguous chunks
                    // before measuring ungapped lengths and ROI overlaps.
                    if let Some(previous) = intervals.last_mut()
                        && previous.left().end() == left.start()
                        && right.end() == previous.right().start()
                    {
                        *previous = InvSegment::new(
                            Interval::new(previous.left().start(), left.end()).unwrap(),
                            Interval::new(right.start(), previous.right().end()).unwrap(),
                        )
                        .unwrap();
                    } else {
                        intervals.push(InvSegment::new(left, right).unwrap());
                    }
                }
                alignment::Op::Mismatch | alignment::Op::GapSecond | alignment::Op::GapFirst => {}
                alignment::Op::Match => {
                    unreachable!()
                }
            }
        }

        let mut stats = EquivRunStats::default();
        for segment in &intervals {
            stats.all.max_len = stats.all.max_len.max(segment.len());
            stats.all.total_len += segment.len();
            for arm in [segment.left(), segment.right()] {
                for (roi, _) in rois.query(*arm) {
                    let overlap = arm.intersection_length(&roi);
                    stats.in_roi.max_len = stats.in_roi.max_len.max(overlap);
                    stats.in_roi.total_len += overlap;
                }
            }
        }

        if intervals.is_empty() || !filter.is_valid(aln.score(), &stats) {
            continue;
        }

        candidates.push(Wrapper(aln, intervals));
    }

    // Construct the final list of inverted repeats
    let passed = suppress_overlaps(candidates);
    let (mut irs, mut scores) = (
        Vec::with_capacity(passed.len()),
        Vec::with_capacity(passed.len()),
    );
    for wrapper in passed {
        irs.push(InvRepeat::new(wrapper.1).unwrap());
        scores.push(*wrapper.0.score());
    }

    Ok((irs, scores))
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    #[ignore] // The current implementation is faulty and actually overwrites
    // a lower scoring but valid hit with a higher scoring but invalid hit
    // It will be fixed in a future version
    fn match_filter_keeps_lower_scoring_valid_repeat() -> Result<()> {
        // A higher scoring hit that doesn't pass filters shouldn't overwrite
        // a lower scoring but valid hit
        let mut filter = Filter::<i32>::default();
        filter.set_min_matches(7, 0);

        let expected = InvRepeat::new(vec![
            InvSegment::new((0..6).try_into()?, (9..15).try_into()?)?,
            InvSegment::new((7..8).try_into()?, (8..9).try_into()?)?,
        ])?;
        let (repeats, scores) = run(b"UUCCUAGCGUGGGAG", filter, Scoring::default())?;

        assert_eq!(scores, [2]);
        assert_eq!(repeats, [expected]);
        Ok(())
    }

    #[test]
    fn roi_overlap_counts_both_arms_at_the_correct_positions() -> Result<()> {
        for (seq, roi, total, ungapped, score) in [
            (b"AU".as_slice(), 1..2, 1, 1, 1),
            (b"AAUU".as_slice(), 0..4, 4, 2, 2),
        ] {
            let mut filter = Filter::<i32>::default();
            filter.set_rois(vec![roi.try_into()?]);
            for (min_total, min_ungapped, expected) in [
                (total, ungapped, vec![score]),
                (total + 1, ungapped, vec![]),
                (total, ungapped + 1, vec![]),
            ] {
                filter.set_min_roi_overlap(min_total, min_ungapped);
                let (repeats, scores) = run(seq, filter.clone(), Scoring::default())?;
                assert_eq!(repeats.len(), expected.len());
                assert_eq!(
                    scores, expected,
                    "ROI thresholds ({min_total}, {min_ungapped})"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn long_match_runs_are_not_split_by_traceback_chunks() -> Result<()> {
        let seq = [vec![b'A'; 256], vec![b'U'; 256]].concat();
        let mut filter = Filter::<i32>::default();
        filter
            .set_min_score(256)
            .set_min_matches(256, 256)
            .set_rois(vec![Interval::new(0, seq.len())?])
            .set_min_roi_overlap(512, 256);
        let (repeats, scores) = run(&seq, filter.clone(), Scoring::default())?;
        assert_eq!(scores, [256]);
        assert_eq!(repeats[0].segments().len(), 1);
        assert_eq!(repeats[0].len(), 256);
        filter.set_min_matches(257, 256);
        assert!(run(&seq, filter, Scoring::default())?.0.is_empty());
        Ok(())
    }

    #[test]
    fn input_is_normalized_and_validated() -> Result<()> {
        let filter = Filter::<i32>::default();
        let rna = run(b"ACGNNCGU", filter.clone(), Scoring::default())?;
        assert!(rna.1.contains(&3));
        for seq in [b"acgnncgu", b"aCgNnCgU", b"ACGNNCGT", b"acgnncgt"] {
            assert_eq!(run(seq, filter.clone(), Scoring::default())?, rna);
        }
        for base in [b'R', b'X', b'-', b' ', b'\n', 0, 0xFF] {
            let error = run(&[b'A', base, b'U'], filter.clone(), Scoring::default())
                .unwrap_err()
                .to_string();
            assert!(error.contains(&format!("0x{base:02X} at index 1")));
        }
        Ok(())
    }

    #[test]
    fn invalid_gap_scores_are_rejected_even_for_short_sequences() {
        for (gap_open, gap_extend) in [(-1, -2), (0, -1), (-5, 1), (1, 1)] {
            let scoring = Scoring {
                complementary: 3i32,
                mismatch: -10,
                gap_open,
                gap_extend,
            };
            for seq in [b"".as_slice(), b"A", b"AU", b"AANNCCGGUU"] {
                let error = run(seq, Filter::default(), scoring)
                    .unwrap_err()
                    .to_string();
                assert!(error.contains("gap_open <= gap_extend <= 0"));
                assert!(error.contains(&format!("gap_open={gap_open}, gap_extend={gap_extend}")));
            }
        }
    }

    #[test]
    fn valid_gap_scores_preserve_affine_scoring() -> Result<()> {
        let expected = InvRepeat::new(vec![
            InvSegment::new((0..2).try_into()?, (8..10).try_into()?)?,
            InvSegment::new((4..6).try_into()?, (6..8).try_into()?)?,
        ])?;
        for (gap_open, gap_extend) in [(-2, -1), (-1, -1), (-1, 0), (0, 0)] {
            let scoring = Scoring {
                complementary: 3i32,
                mismatch: -10,
                gap_open,
                gap_extend,
            };
            // Four complementary pairs separated by a two-base gap.
            let expected_score = 4 * scoring.complementary + gap_open + gap_extend;
            let mut filter = Filter::default();
            filter.set_min_score(expected_score);
            let (repeats, scores) = run(b"AANNCCGGUU", filter, scoring)?;
            assert_eq!(scores, [expected_score]);
            assert_eq!(repeats.as_slice(), std::slice::from_ref(&expected));
        }
        Ok(())
    }

    #[test]
    fn match_thresholds_filter_predictions() -> Result<()> {
        for (total, ungapped, expected) in [
            (0, 0, vec![1, 1, 2]),
            (2, 2, vec![2]),
            (3, 2, vec![]),
            (2, 3, vec![]),
        ] {
            let mut filter = Filter::<i32>::default();
            filter.set_min_matches(total, ungapped);
            let (repeats, mut scores) = run(b"AAUU", filter, Scoring::default())?;
            scores.sort_unstable();
            assert_eq!(repeats.len(), expected.len());
            assert_eq!(scores, expected, "match thresholds ({total}, {ungapped})");
        }
        Ok(())
    }

    #[test]
    fn suppression_keeps_the_best_paths_in_every_input_order() -> Result<()> {
        // The first two paths are disjoint; the third intersects both.
        // A weaker bridge loses to both, while a stronger bridge excludes both.
        for (bridge_score, expected) in [(1, vec![2, 3]), (4, vec![4])] {
            for order in [(2, 0usize..2), (3, 2..4), (bridge_score, 1..4)]
                .into_iter()
                .permutations(3)
            {
                let mut candidates = Vec::new();
                for (score, range) in order {
                    let step = alignment::Step::new(alignment::Op::Equivalent, range.len() as u8)?;
                    let aln = Alignment::new(score, vec![step], range.clone(), range);
                    candidates.push(Wrapper(aln, Vec::new()));
                }
                let mut scores: Vec<_> = suppress_overlaps(candidates)
                    .iter()
                    .map(|hit| *hit.0.score())
                    .collect();
                scores.sort();
                assert_eq!(scores, expected, "bridge score {bridge_score}");
            }
        }
        Ok(())
    }

    #[test]
    fn gapped_repeat_has_the_expected_fold() -> Result<()> {
        let seq = b"UUGUAUUAUAUCUAUGUAUAUAGAUAUNAAUACGA";
        let mut filter = Filter::<i32>::default();
        filter.set_min_score(12);

        let expected = InvRepeat::new(vec![
            InvSegment::new(Interval::new(0, 7)?, Interval::new(28, 35)?)?,
            InvSegment::new(Interval::new(7, 17)?, Interval::new(17, 27)?)?,
        ])?;
        let (repeats, scores) = run(seq, filter, Scoring::default())?;
        assert_eq!(scores, [12]);
        assert_eq!(repeats, [expected]);
        Ok(())
    }
}
