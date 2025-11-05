use std::collections::HashMap;
use std::collections::hash_map::Entry;

use derive_getters::Dissolve;

use biobit_alignment_rs::pairwise::{
    scoring,
    sw::algo::{BestOrientationTracer, GapTracer, Tracer},
    sw::storage::{AlignmentSeed, Storage},
};
use biobit_core_rs::loc::{Interval, IntervalOp};

use super::alignment::{CandidatesTracker, MayBeLocallyOptimal};
use super::filtering::SoftFilter;

// An efficient algorithm to locate all locally optimal alignments between two sequences allowing for gaps
// DOI: 10.1093/bioinformatics/9.6.729

pub struct ROITracker {
    intervals: Vec<Interval<usize>>,
    // Main index of the current interval in the list of ROIs. Goes from 0 to intervals.len().
    main: usize,
    // Secondary index of the current interval in the list of ROIs. Goes from intervals.len() to 0.
    secondary: usize,
}

impl ROITracker {
    pub fn new(mut intervals: Vec<Interval<usize>>) -> Self {
        intervals.sort();
        Self {
            intervals,
            main: 0,
            secondary: 0,
        }
    }

    pub fn main(&mut self, newpos: usize) -> bool {
        // Reset the secondary index
        self.secondary = self.intervals.len();

        // Move the main index to the right if the current interval is already passed
        while self.main < self.intervals.len() && self.intervals[self.main].end() <= newpos {
            self.main += 1;
        }
        debug_assert!(
            self.main == self.intervals.len()
                || self.intervals[self.main].start() >= newpos
                || self.intervals[self.main].start() <= newpos
                    && newpos < self.intervals[self.main].end()
        );

        self.main < self.intervals.len() && self.intervals[self.main].contains(newpos)
    }

    pub fn secondary(&mut self, newpos: usize) -> bool {
        // Move the secondary index to the left if the current interval is already passed
        while self.secondary > 0 && self.intervals[self.secondary - 1].start() > newpos {
            self.secondary -= 1;
        }
        debug_assert!(
            self.secondary == 0
                || self.intervals[self.secondary - 1].end() <= newpos
                || self.intervals[self.secondary - 1].start() <= newpos
                    && newpos < self.intervals[self.secondary - 1].end()
        );

        self.secondary > 0 && self.intervals[self.secondary - 1].contains(newpos)
    }
}

#[derive(Dissolve)]
pub struct AllOptimal<S: scoring::Score, F: SoftFilter<Score = S>> {
    // Filtering for the alignment candidates
    filter: F,

    // ROIs tracking functionality
    rois: ROITracker,
    is_roi: bool,

    // Main cache
    diagonal: Option<CandidatesTracker<S>>,
    cache: Vec<Option<CandidatesTracker<S>>>,

    // Gapped paths caches
    best_gap_row: Option<CandidatesTracker<S>>,
    best_gap_col: Vec<Option<CandidatesTracker<S>>>,

    // Cache for finished paths in each row
    results: Vec<HashMap<usize, MayBeLocallyOptimal<S>>>,
}

impl<S: scoring::Score, F: SoftFilter<Score = S>> AllOptimal<S, F> {
    pub fn new(filter: F, rois: Vec<Interval<usize>>) -> Self {
        Self {
            filter,
            rois: ROITracker::new(rois),
            is_roi: false,
            diagonal: None,
            cache: vec![None; 128],
            best_gap_row: None,
            best_gap_col: vec![None; 128],
            results: vec![HashMap::new(); 128],
        }
    }

    fn save(&mut self, candidate: CandidatesTracker<S>) {
        // If path is not valid - skip it completely
        if !self.filter.is_valid(&candidate.score, &candidate.stats) {
            return;
        }

        let row = candidate.start.0;
        let col = candidate.start.1;

        match self.results[row].entry(col) {
            Entry::Occupied(mut existing) => {
                if existing.get().score < candidate.score {
                    existing.insert(candidate.into());
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(candidate.into());
            }
        }
    }

    fn update_diagonal(&mut self, newdiag: Option<CandidatesTracker<S>>) {
        // Try to save the previous diagonal if it wasn't consumed before
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };

        // Update the diagonal
        self.diagonal = newdiag;
    }
}

impl<S: scoring::Score, F: SoftFilter<Score = S>> BestOrientationTracer for AllOptimal<S, F> {
    type Score = S;

    #[inline(always)]
    fn gap_row(&mut self, row: usize, _: usize, _: S) {
        let newdiag = self.cache[row].take();

        // debug_assert!(self.best_gap_row.as_ref().unwrap().end == (row, col));
        // debug_assert!(self.best_gap_row.as_ref().unwrap().score == score);
        self.cache[row] = self.best_gap_row.clone();

        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, _: usize, _: S) {
        let newdiag = self.cache[row].take();

        // debug_assert!(self.best_gap_col[row].as_ref().unwrap().end == (row, col));
        // debug_assert!(self.best_gap_col[row].as_ref().unwrap().score == score);
        self.cache[row] = self.best_gap_col[row].clone();

        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: S) {
        let newdiag = self.cache[row].take();
        let is_roi = self.is_roi || self.rois.secondary(self.results.len() - row);

        match self.diagonal.take() {
            None => {
                // Start the new path
                self.cache[row] = Some(CandidatesTracker::new((row, col), score, is_roi));
            }
            Some(mut diagonal) => {
                // Extend & consume the diagonal
                diagonal.equivalent(row, col, score, is_roi, &self.filter);
                self.cache[row] = Some(diagonal);
            }
        }
        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn none(&mut self, row: usize, _: usize) {
        let newdiag = self.cache[row].take();
        self.update_diagonal(newdiag);
    }
}

impl<S: scoring::Score, F: SoftFilter<Score = S>> GapTracer for AllOptimal<S, F> {
    type Score = S;

    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, score: S) {
        self.best_gap_row = match &self.cache[row - 1] {
            None => {
                unreachable!()
            }
            Some(x) => {
                let mut x = x.clone();
                x.gap(row, col, score, &self.filter);
                Some(x)
            }
        }
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, score: S) {
        match &mut self.best_gap_row {
            None => {
                unreachable!()
            }
            Some(x) => {
                x.gap(row, col, score, &self.filter);
            }
        }
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, score: S) {
        self.best_gap_col[row] = match &self.cache[row] {
            None => {
                unreachable!()
            }
            Some(x) => {
                let mut x = x.clone();
                x.gap(row, col, score, &self.filter);
                Some(x)
            }
        };
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, score: S) {
        match &mut self.best_gap_col[row] {
            None => {
                unreachable!()
            }
            Some(x) => {
                x.gap(row, col, score, &self.filter);
            }
        }
    }
}

impl<S: scoring::Score, F: SoftFilter<Score = S>> Tracer for AllOptimal<S, F> {
    type Score = S;

    fn first_col_start(&mut self) {
        self.is_roi = self.rois.main(0);
    }
    fn first_col_end(&mut self) {
        self.diagonal = None;
    }

    fn col_start(&mut self, col: usize) {
        self.is_roi = self.rois.main(col);
    }
    fn col_end(&mut self, _: usize) {
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };
        self.diagonal = None;
        self.best_gap_row = None;
    }
}

impl<S: scoring::Score, F: SoftFilter<Score = S>> Storage for AllOptimal<S, F> {
    fn reset(&mut self, newrows: usize, _: usize) {
        self.cache.clear();
        self.diagonal = None;
        self.best_gap_row = None;

        // TODO: reuse result vectors when possible
        self.results.clear();
        self.results.resize(newrows, HashMap::new());

        self.cache.clear();
        self.cache.resize(newrows, None);

        self.best_gap_col.clear();
        self.best_gap_col.resize(newrows, None);
    }

    fn finalize(&mut self) -> Vec<AlignmentSeed<S>> {
        {
            let mut cache = Vec::new();
            std::mem::swap(&mut cache, &mut self.cache);

            for x in &mut cache {
                match x.take() {
                    None => {}
                    Some(x) => self.save(x),
                }
            }
            std::mem::swap(&mut cache, &mut self.cache);
        }

        self.results
            .iter()
            .flat_map(|map| {
                map.values().map(|x| AlignmentSeed {
                    row: x.end.0,
                    col: x.end.1,
                    score: x.score,
                })
            })
            .collect()
    }
}
