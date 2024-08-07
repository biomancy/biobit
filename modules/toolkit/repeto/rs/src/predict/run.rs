use eyre::Result;

use biobit_alignment_rs::alignable::Reversed;
use biobit_alignment_rs::pairwise::{alignment, scoring, sw};
use biobit_collections_rs::interval_tree::{Builder, ITree, LapperBuilder};
use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{AsSegment, Segment};

use crate::predict::filtering::Filter;
use crate::predict::scoring::Scoring;
use crate::predict::storage::AllOptimal;
use crate::repeats::{InvRepeat, InvSegment};

use super::storage::filtering::{EquivRunStats, SoftFilter};

pub fn run<S: scoring::Score>(
    seq: &[u8],
    filter: Filter<S>,
    scoring: Scoring<S>,
) -> Result<(Vec<InvRepeat<usize>>, Vec<S>)> {
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
    let alignments = aligner.scan_up_triangle(&Reversed::new(&seq), &seq, 1);

    // During the alignment all overlaps between ROIs and alignments are judged based on diagonal
    // runs that can be either matches or mismatches. However, the threshold for ROI overlaps
    // assumed that the overlap calculated over matches only. Therefore, we need to recalculate
    // the overlap and filter out the alignments that do not meet the criteria.
    let (rois, filter) = {
        let (_, _, tracer) = aligner.dissolve();
        let (allopt, _, _) = tracer.dissolve();
        let filter = allopt.dissolve().0;

        let mut index = LapperBuilder::new();
        for roi in filter.rois() {
            index = index.add(roi, ());
        }
        (index.build(), filter)
    };

    // Alignments are not guaranteed to be non-overlapping so we need to employ additional filtering
    // to suppress suboptimal overlapping alignments.
    let mut passed: Vec<(alignment::Alignment<_, _, _, _>, _)> = Vec::new();

    for alignment in alignments {
        let mut segments = Vec::with_capacity(alignment.steps().len());
        let mut stats = EquivRunStats::default();

        for step in alignment.tracked_steps() {
            match step.step.op() {
                alignment::Op::Equivalent => {
                    stats.all.max_len = stats.all.max_len.max(*step.step.len() as usize);
                    stats.all.total_len += *step.step.len() as usize;

                    let left =
                        Segment::new(step.start.seq2, step.start.seq2 + *step.step.len() as usize)
                            .unwrap();
                    let right = Segment::new(
                        seq.len() - step.start.seq1 - *step.step().len() as usize,
                        seq.len() - step.start.seq1,
                    )
                    .unwrap();

                    for seg in [&left, &right] {
                        let mut iter = rois.intersection(seg);
                        while let Some((overlap, _)) = iter.next() {
                            stats.in_roi.max_len = stats.in_roi.max_len.max(overlap.len());
                            stats.in_roi.total_len += overlap.len();
                        }
                    }

                    segments.push(InvSegment::new(left, right).unwrap());
                }
                alignment::Op::Mismatch | alignment::Op::GapSecond | alignment::Op::GapFirst => {}
                alignment::Op::Match => {
                    unreachable!()
                }
            }
        }

        if segments.is_empty() || !filter.is_valid(&alignment.score(), &stats) {
            continue;
        }

        // Try to insert the alignment into the final list
        let mut intersects_with = usize::MAX;
        for ind in 0..passed.len() {
            if passed[ind].0.intersects(&alignment) {
                intersects_with = ind;
                break;
            }
        }

        if intersects_with == usize::MAX {
            // If the alignment is unique then add it to the list
            passed.push((alignment, segments));
        } else {
            // If the alignment intersects with another alignment then we need to decide which one
            // to keep. The decision is based on the alignment score.
            if passed[intersects_with].0.score() < alignment.score() {
                passed[intersects_with] = (alignment, segments);
            }
        }
    }

    // Construct the final list of inverted repeats
    let (mut irs, mut scores) = (
        Vec::with_capacity(passed.len()),
        Vec::with_capacity(passed.len()),
    );

    for (alignment, segments) in passed {
        irs.push(InvRepeat::new(segments).unwrap());
        scores.push(*alignment.score());
    }

    Ok((irs, scores))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain() -> Result<()> {
        let seq = b"AANNUU";
        let (ir, mut scores) = run(seq, Filter::<i32>::default(), Scoring::default())?;
        assert_eq!(ir.len(), 3);

        scores.sort();
        assert_eq!(scores, vec![1, 1, 2]);

        Ok(())
    }
}
