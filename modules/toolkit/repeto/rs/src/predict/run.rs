use std::ops::Range;

use biobit_alignment::alignable::Reversed;
use biobit_alignment::pairwise::{alignment, scoring, sw};

use crate::predict::filtering::Filters;
use crate::predict::scoring::Scoring;
use crate::predict::storage::AllOptimal;
use crate::repeats::inv;

pub fn run<S: scoring::Score>(
    seq: &[u8],
    filters: Filters<S>,
    scoring: Scoring<S>,
) -> (Vec<inv::Repeat<isize>>, Vec<S>) {
    let scoring = scoring::compose(
        scoring::symbols::Equality::new(scoring.complementary, scoring.mismatch),
        scoring::gaps::Affine { open: scoring.gap_open, extend: scoring.gap_extend },
        scoring::equiv::RNAComplementarity {},
    );

    let mut aligner = sw::Engine::new(
        AllOptimal::new(thr, params.rois),
        sw::traceback::TraceMatrix::new(), scoring,
    );

    // sequence orientation:
    // ------>
    // ^    /
    // |   /
    // |  /
    // | /
    // |/
    let alignments = aligner.scan_up_triangle(
        &Reversed::new(seq), &seq, 1,
    );

    // Collapse overlapping paths
    // let mut nms = Vec::with_capacity(buffer.len() / 2);
    // for candidate in buffer {
    //     let intersects = false;
    //     for algn in nms.iter_mut() {
    //
    //     }
    //     if !intersects {
    //         nms.push(candidate)
    //     }
    // }
    //
    // // (It's not guaranteed that AllOpt will produce non overlapping paths)
    // let mut alignments: Vec<alignment::Alignment<i64>> = Vec::with_capacity(128);
    // for item in buffer.into_iter().rev() {
    //     let intersects = alignments.iter().any(|x| x.intersects(&item));
    //     if !intersects {
    //         alignments.push(item);
    //     }
    // }

    // Convert to segments & inverted repeats
    let mut scores = Vec::with_capacity(alignments.len());
    let alignments = alignments.into_iter().filter_map(|x| {
        let mut segments = Vec::with_capacity(x.steps.len());

        let mut max_matches_run = 0;
        for step in x.coalesced_steps() {
            match step.op {
                alignment::Op::Equivalent => { unreachable!() }
                alignment::Op::Match => {
                    max_matches_run = max_matches_run.max(step.len);

                    let left = Range {
                        start: (step.start.seq2) as isize,
                        end: (step.start.seq2 + step.len) as isize,
                    };
                    let right = Range {
                        start: (seq.len() - step.start.seq1 - step.len) as isize,
                        end: (seq.len() - step.start.seq1) as isize,
                    };
                    segments.push(inv::Segment::new(left, right));
                }
                alignment::Op::Mismatch | alignment::Op::GapSecond | alignment::Op::GapFirst => {}
            }
        }
        scores.push(x.score);
        Some(inv::Repeat::new(segments))
    }).collect();
    (alignments, scores)
}
