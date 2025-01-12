use eyre::Result;
use rstar::{Envelope, RTree, RTreeObject, SelectionFunction, AABB};

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

struct Wrapper<S: scoring::Score>(
    usize,
    Alignment<S, u8, usize, usize>,
    Vec<InvSegment<usize>>,
);

impl<S: scoring::Score> RTreeObject for Wrapper<S> {
    type Envelope = AABB<(isize, isize)>;

    fn envelope(&self) -> Self::Envelope {
        let seq1 = self.1.seq1();
        let seq2 = self.1.seq2();

        AABB::from_corners(
            (seq1.start as isize, seq2.start as isize),
            (seq1.end as isize, seq2.end as isize),
        )
    }
}

struct SelectByID(AABB<(isize, isize)>, usize);

impl<S: scoring::Score> SelectionFunction<Wrapper<S>> for SelectByID {
    fn should_unpack_parent(&self, envelope: &AABB<(isize, isize)>) -> bool {
        envelope.contains_envelope(&self.0)
    }

    fn should_unpack_leaf(&self, leaf: &Wrapper<S>) -> bool {
        leaf.0 == self.1
    }
}

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
            index = index.addi(roi, ());
        }
        (index.build(), filter)
    };

    // Alignments are not guaranteed to be non-overlapping so we need to employ additional filtering
    // to suppress suboptimal overlapping alignments.

    let mut passed: RTree<Wrapper<_>> = RTree::new();
    let mut length = 0;
    for aln in alignments.into_iter() {
        let mut intervals = Vec::with_capacity(aln.steps().len());
        let mut stats = EquivRunStats::default();

        for step in aln.tracked_steps() {
            match step.step.op() {
                alignment::Op::Equivalent => {
                    stats.all.max_len = stats.all.max_len.max(*step.step.len() as usize);
                    stats.all.total_len += *step.step.len() as usize;

                    let left =
                        Interval::new(step.start.seq2, step.start.seq2 + *step.step.len() as usize)
                            .unwrap();
                    let right = Interval::new(
                        seq.len() - step.start.seq1 - (*step.step().len() as usize),
                        seq.len() - step.start.seq1,
                    )
                    .unwrap();

                    for seg in [&left, &right] {
                        for (overlap, _) in rois.query(seg.start(), seg.end()) {
                            let overlap = seg.intersection_length(&overlap);
                            stats.in_roi.max_len = stats.in_roi.max_len.max(overlap);
                            stats.in_roi.total_len += overlap;
                        }
                    }

                    intervals.push(InvSegment::new(left, right).unwrap());
                }
                alignment::Op::Mismatch | alignment::Op::GapSecond | alignment::Op::GapFirst => {}
                alignment::Op::Match => {
                    unreachable!()
                }
            }
        }

        if intervals.is_empty() || !filter.is_valid(aln.score(), &stats) {
            continue;
        }

        // Look for overlaps with existing hits
        let mut intersection = None;
        let envelope = AABB::from_corners(
            (aln.seq1().start as isize, aln.seq2().start as isize),
            (aln.seq1().end as isize, aln.seq2().end as isize),
        );
        for existing in passed.locate_in_envelope_intersecting(&envelope) {
            if existing.1.intersects(&aln) {
                intersection = Some(SelectByID(envelope, existing.0));
                break;
            }
        }

        match intersection {
            Some(x) => {
                let ind = x.1;
                passed.remove_with_selection_function(x);
                passed.insert(Wrapper(ind, aln, intervals));
            }
            None => {
                passed.insert(Wrapper(length, aln, intervals));
                length += 1;
            }
        }
    }

    // Construct the final list of inverted repeats
    let (mut irs, mut scores) = (Vec::with_capacity(length), Vec::with_capacity(length));
    for wrapper in passed.drain() {
        irs.push(InvRepeat::new(wrapper.2).unwrap());
        scores.push(*wrapper.1.score());
    }

    Ok((irs, scores))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain() -> Result<()> {
        let seq = b"AAUU";
        let (ir, mut scores) = run(seq, Filter::<i32>::default(), Scoring::default())?;
        assert_eq!(ir.len(), 3);

        scores.sort();
        assert_eq!(scores, vec![1, 1, 2]);

        Ok(())
    }

    // #[test]
    // fn why_do_i_fail() -> Result<()> {
    //     let seq = b"UUGUAUUAUAUCUAUGUAUAUAGAUAUNAAUACGA";
    //
    //     let mut filter = Filter::<i32>::default();
    //     filter.set_min_score(12);

        // for ind in 0..seq.len() {
        //     let mut seq = seq.to_vec();
        //     seq.remove(ind);
        //     println!("ind: {:?}, {}", (ind,), String::from_utf8(seq.clone())?);
        //     run(&seq, filter.clone(), Scoring::default())?;
        //
        //     for j in ind..seq.len() {
        //         if j >= seq.len() {
        //             break;
        //         }
        //
        //         println!("ind: {:?}, {}", (ind,), String::from_utf8(seq.clone())?);
        //         run(&seq, filter.clone(), Scoring::default())?;
        //     }
        // }

    //     run(seq, filter, Scoring::default())?;
    //     Ok(())
    // }

    // #[test]
    // fn why_there_is_no_dsRNA() -> Result<()> {
    //     let seq = b"GUGGCUCAUGCCUGCAGUCCCAGCACUUUGGGAGGCUGAGGCAGGUGUAUCACCUGAGGUCAGGAGUUCGAGACCAGCCUGGCCAACAUGGUGAAACCCUGUUUCCACGAAAAAUACAAUAAACUAGCUGGGCAUGGUGGUACGUGCCUGUAAUCCCAGCUACUUGGGAGGCUGAGACACGAGAAUCGCUUGAACCUGGGAGGCAGAGGUUGUAGUGAGCCGAGAUUGCGCCACUGCACUCCAGCCUGGGUGACAGAGCGAGACUCCAUCUCAAAAAUAAAUAUAUAAAAUAAAAUUGAGAUAUAGUUCAGAAAGCCCACCAAGAUCUGAAUUAUUUAAACCUGUGUCCAAAUUGUUUUUGUUCUCAUUAUCUUGCAAUUGUUUUUCUUUGCAUACAGGCUCGUGAGCCCUUGGUUGUGUUUCUCCCUUUUUUCUCUCACUGUUUUUUCUCUUUUCCUUUUUGAGACGGGUCUCACUCUGUUGCCCAGGCUGGCAUGCAGUGACACAGUCAUAGCUCACUGCAGCCUCAGCCUCAACCUUCCAGGCUCAAGCGAUCCUCCGACCUCAGCCUCCAAAGUAGCUGGGACUACUGCUGUGCGACACCAUGCCUGGCUAAUUUUUGAAUUUUUAUUUUUAGAGAUGGGGUCUCCCUAUUUUGGCCAGUCUGGUCUCAAACUCCUGGGCUCAAGAGAUCCUCCAGCCUCGGCCUCCCAAAGUGCUGAGAUUACAGGUGUGAGCCACUGU";
    //
    //     let mut filter = Filter::<i32>::default();
    //     filter.set_min_score(12);
    //     filter.set_min_roi_overlap(1, 1);
    //     filter.set_rois(vec![Interval::new(469, 546).unwrap()]);
    //
    //     run(seq, filter, Scoring::default())?;
    //     Ok(())
    // }
}
