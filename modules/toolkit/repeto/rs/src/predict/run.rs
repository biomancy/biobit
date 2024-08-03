use biobit_alignment_rs::alignable::Reversed;
use biobit_alignment_rs::pairwise::{alignment, scoring, sw};
use biobit_core_rs::loc::Segment;

use crate::predict::filtering::Filter;
use crate::predict::scoring::Scoring;
use crate::predict::storage::AllOptimal;
use crate::repeats::inv;

pub fn run<S: scoring::Score>(
    seq: &[u8],
    filter: Filter<S>,
    scoring: Scoring<S>,
) -> (Vec<inv::Repeat<isize>>, Vec<S>) {
    let scoring = scoring::compose(
        scoring::symbols::Equality::new(scoring.complementary, scoring.mismatch),
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

    // // It's not guaranteed that AllOpt will produce non overlapping paths
    // let mut alignments = Vec::with_capacity(128);
    // for item in all_alignments.into_iter().rev() {
    //     let intersects = alignments
    //         .iter()
    //         .any(|x| x.intersects(&item));
    //     if !intersects {
    //         alignments.push(item);
    //     }
    // }

    // Convert to segments & inverted repeats
    let mut scores = Vec::with_capacity(alignments.len());
    let alignments = alignments
        .into_iter()
        .filter_map(|x| {
            let mut segments = Vec::with_capacity(x.steps().len());

            let mut max_matches_run = 0;
            for step in x.tracked_steps() {
                match step.step.op() {
                    alignment::Op::Equivalent => {
                        unreachable!()
                    }
                    alignment::Op::Match => {
                        max_matches_run = max_matches_run.max(*step.step.len() as isize);

                        let left = Segment::new(
                            step.start.seq2 as isize,
                            step.start.seq2 as isize + *step.step.len() as isize,
                        )
                        .unwrap();
                        let right = Segment::new(
                            (seq.len() - step.start.seq1 - *step.step().len() as usize) as isize,
                            (seq.len() - step.start.seq1) as isize,
                        )
                        .unwrap();
                        segments.push(inv::InvSegments::new(left, right).unwrap());
                    }
                    alignment::Op::Mismatch
                    | alignment::Op::GapSecond
                    | alignment::Op::GapFirst => {}
                }
            }
            scores.push(*x.score());
            Some(inv::Repeat::new(segments).unwrap())
        })
        .collect();
    (alignments, scores)
}
