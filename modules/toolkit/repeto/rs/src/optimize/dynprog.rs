use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::ops::Range;

use itertools::Itertools;

use biobit_core_rs::loc::{AsSegment, Segment};
use biobit_core_rs::num::PrimInt;

use super::{index, trace};
use super::inv;
use super::trace::TraceCell;

// General rules:
// 1. If a hypothesis is included -> the best subset of its embedded hypothesis is also included
// 2. If a hypothesis is excluded -> some of its embedded hypothesis might be included
// Hypothesis => predicted InvertedRepeat
//
// We need to consdider the matrix with start/ends and the following recursive equation:
// f(s,e) = max of the following
// * f(s, e - 1)
// * max of the following:
//   * f(s, start(RNA)) + weight(RNA) + sum[f(start(gap_i), end(gap_i)) for all gaps in RNA where end(RNA) == e]
struct Workload<'a, Idx, IR, Score>
where
    Idx: PrimInt,
    IR: Borrow<inv::Repeat<Idx>>,
    Score: PrimInt,
{
    pub index: index::Index<Idx>,
    pub invrep: &'a [IR],
    pub scores: &'a [Score],
}

pub struct DynProgSolution<Score: PrimInt> {
    pub tracer: trace::Tracer,
    pub cache: Vec<BTreeMap<usize, Score>>,
}

impl<Score: PrimInt> DynProgSolution<Score> {
    pub fn new() -> Self {
        Self {
            tracer: trace::Tracer::new(0, 0),
            cache: vec![],
        }
    }

    pub fn solve<Idx, T>(&mut self, invrep: &[T], scores: &[Score]) -> (Vec<usize>, Score)
    where
        Idx: PrimInt,
        T: Borrow<inv::Repeat<Idx>>,
    {
        debug_assert!(invrep.len() == scores.len());
        let w = Workload {
            index: index::Index::new(invrep),
            invrep,
            scores,
        };

        // Prepare caches & the tracer
        self.cache
            .resize(w.index.starts().len(), Default::default());
        self.tracer
            .reset(w.index.starts().len(), w.index.ends().len());

        // Solve in a limited fashion to reduce the total number of recursive calls
        for i in 0..w.index.ends().len() {
            self.subsolve(&w, 0, i);
        }
        let score = self.subsolve(&w, 0, w.index.ends().len() - 1);
        let optimum = self.tracer.trace(0, w.index.ends().len() - 1);
        (optimum, score)
    }

    fn subsolve<Idx, IR>(&mut self, w: &Workload<Idx, IR, Score>, sind: usize, eind: usize) -> Score
    where
        Idx: PrimInt,
        IR: Borrow<inv::Repeat<Idx>>,
    {
        // Sanity check
        debug_assert!(sind <= w.index.starts().len() && eind <= w.index.ends().len());

        // Prevent recursion for incorrect intervals
        if w.index.starts()[sind].pos >= w.index.ends()[eind].pos {
            return Score::zero();
        }

        // Cached lookup
        if let Some(score) = self.cache[sind].get(&eind) {
            return *score;
        }

        let mut bestt = None;

        // DP equation
        // * We skip the current end <- the best option is to use the previous end
        if eind > 0 {
            let score = self.subsolve(w, sind, eind - 1);
            if score > Score::zero() {
                bestt = Some((
                    trace::TraceCell {
                        rnaid: None,
                        included: vec![(sind, eind - 1)],
                    },
                    score,
                ))
            }
        }

        // * We look for the best rnafold here
        // TODO: please borrow checker without doing clone here
        for &rnaid in &w.index.ends()[eind].repeats.clone() {
            let (rnasind, rnaeind) = w.index.revmap(rnaid);
            // Skip rnas that are not inside the current region
            if rnasind < sind {
                continue;
            }

            let rna = &w.invrep[rnaid];
            let mut score = w.scores[rnaid];

            debug_assert!(
                rnasind >= sind
                    && rnaeind == eind
                    && w.index.starts()[sind].pos <= rna.borrow().brange().start()
                    && w.index.ends()[eind].pos >= rna.borrow().brange().end()
            );

            // Include the best combination of 'embeddable' RNAs
            let (gscore, mut trace) = self.gapsolve(w, w.index.blocks(rnaid), rnasind, rnaeind);
            score = score + gscore;

            // Find the closest end that doesn't contain the current rnafold
            let mut preeind =
                index::bisect::right(w.index.ends(), rna.borrow().brange().start(), 0, eind);
            if preeind != 0 {
                preeind -= 1;
                debug_assert!(rna.borrow().brange().start() >= w.index.ends()[preeind].pos);

                // Can we include it?
                if w.index.ends()[preeind].pos > w.index.starts()[sind].pos {
                    let pscore = self.subsolve(w, sind, preeind);
                    if pscore > Score::zero() {
                        score = score + pscore;
                        trace.push((sind, preeind));
                    }
                }
            }

            if score > Score::zero() && (bestt.is_none() || bestt.as_ref().unwrap().1 < score) {
                bestt = Some((
                    TraceCell {
                        rnaid: Some(rnaid),
                        included: trace,
                    },
                    score,
                ));
            }
        }

        return match bestt {
            None => {
                // No traces scored > 0
                self.cache[sind].insert(eind, Score::zero());
                Score::zero()
            }
            Some(bestt) => {
                self.tracer.update(sind, eind, bestt.0);
                self.cache[sind].insert(eind, bestt.1);
                bestt.1
            }
        };
    }

    fn gapsolve<Idx, IR>(
        &mut self,
        w: &Workload<Idx, IR, Score>,
        blocks: &[Segment<Idx>],
        mut minsind: usize,
        maxeind: usize,
    ) -> (Score, Vec<(usize, usize)>)
    where
        Idx: PrimInt,
        IR: Borrow<inv::Repeat<Idx>>,
    {
        // Blocks are sorted and for each start-end gap between adjacent blocks we need to find
        // the best possible sind/eind so that: minsind < start(sind) <= start < end <= end(eind) < maxend

        let mut traces = Vec::new();
        let (mut score, mut minend) = (Score::zero(), 0);

        for (prv, nxt) in blocks.iter().tuple_windows() {
            debug_assert!(prv.end() <= nxt.start());

            // No gap
            if prv.end() == nxt.start() {
                continue;
            }
            let gap = Range {
                start: prv.end(),
                end: nxt.start(),
            };

            // Find the closest known interval inside the gap
            let sind =
                index::bisect::left(w.index.starts(), gap.start, minsind, w.index.starts().len());
            let mut eind = index::bisect::right(w.index.ends(), gap.end, minend, maxeind);
            debug_assert!(eind < w.index.ends().len() && sind <= w.index.starts().len());

            // No rnas inside the gap
            if eind == 0 || sind == w.index.starts().len() {
                continue;
            }
            eind -= 1;

            let (start, end) = (w.index.starts()[sind].pos, w.index.ends()[eind].pos);
            debug_assert!(gap.start <= start && end <= gap.end);

            // No valid matches
            if start >= end {
                continue;
            }

            // Find the best combination
            let addition = self.subsolve(w, sind, eind);
            if addition > Score::zero() {
                score = score + addition;
                traces.push((sind, eind));
            }

            // Since the gaps will only move right, we can be sure that:
            // * next gaps will end after the current one
            // * next gaps will start after the current one
            minsind = sind;
            minend = eind;
        }
        debug_assert!(
            score.is_zero() && traces.len() == 0 || score > Score::zero() && traces.len() > 0
        );
        (score, traces)
    }
}
