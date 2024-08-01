use biobit_alignment::pairwise::scoring;
use biobit_alignment::pairwise::sw::algo::{BestDirectionTracer, GapTracer, Tracer};
use biobit_alignment::pairwise::sw::storage::{AlignmentSeed, Storage};

use path::{full, partial};

mod path;

// An efficient algorithm to locate all locally optimal alignments between two sequences allowing for gaps
// 10.1093/bioinformatics/9.6.729

// How to track this stuff? Who knows....
// cumulative-max-current length in a builder

pub struct Thresholds<S: scoring::Score> {
    pub minscore: S,
    pub min_single_stem_length: usize,
    pub min_single_roi_overlap: usize,
}

impl<S: scoring::Score> Thresholds<S> {
    pub fn is_ok(&self, path: &partial::Path<S>) -> bool {
        return if path.score < self.minscore {
            false
        } else if path.optimal.stem.max_length < self.min_single_stem_length {
            false
        } else if path.optimal.roi.max_length < self.min_single_roi_overlap {
            false
        } else {
            true
        };
    }
}

pub struct ROI {
    intervals: Vec<(usize, usize)>,
    pointer: usize,
}

impl ROI {
    pub fn new(mut intervals: Vec<(usize, usize)>) -> Self {
        intervals.sort();
        Self { intervals, pointer: 0 }
    }

    pub fn step(&mut self, newpos: usize) -> bool {
        while self.pointer < self.intervals.len() && self.intervals[self.pointer].1 <= newpos {
            self.pointer += 1;
        }
        debug_assert!(
            self.pointer == self.intervals.len() ||
                self.intervals[self.pointer].0 >= newpos ||
                self.intervals[self.pointer].0 <= newpos && newpos < self.intervals[self.pointer].1
        );

        self.pointer < self.intervals.len() && self.intervals[self.pointer].0 <= newpos

        // Fast-forward to the closest ROI
        // while self.rois.last().is_some_and(|x| x.1 <= nxtcol) {
        //     self.rois.pop();
        // }
        // debug_assert!(self.rois.last().map_or_else(
        //     true, |x| nxtcol < x.0 || x.0 <= nxtcol && nxtcol < x.1,
        // ));
        //
        // self.rois.last().map_or_else(
        //     false, |x| x.0 <= nxtcol,
        // )
    }
}


pub struct AllOptimal<S: scoring::Score> {
    // Thresholds
    thresholds: Thresholds<S>,

    // ROIs
    rois: ROI,
    is_roi: bool,

    // Main cache
    diagonal: Option<partial::Path<S>>,
    cache: Vec<Option<partial::Path<S>>>,

    // Gapped paths caches
    best_gap_row: Option<partial::Path<S>>,
    best_gap_col: Vec<Option<partial::Path<S>>>,

    // Cache for finished paths in each row
    results: Vec<Vec<full::Path<S>>>,
}

impl<S: scoring::Score> AllOptimal<S> {
    pub fn new(thresholds: Thresholds<S>, rois: Vec<(usize, usize)>) -> Self {
        Self {
            thresholds,
            rois: ROI::new(rois),
            is_roi: false,
            diagonal: None,
            cache: vec![None; 128],
            best_gap_row: None,
            best_gap_col: vec![None; 128],
            results: vec![Vec::new(); 128],
        }
    }

    fn save(&mut self, p: partial::Path<S>) {
        if !self.thresholds.is_ok(&p) {
            return;
        }

        let row = p.start.0;
        for r in &mut self.results[row] {
            if r.start == p.start {
                // If match & better score -> update the hit
                if r.score < p.score {
                    *r = p.into();
                }
                return;
            }
        }
        // New match -> store the new path
        self.results[row].push(p.into())
    }

    #[inline(always)]
    fn update_diagonal(&mut self, newdiag: Option<partial::Path<S>>) {
        // Try to save the previous diagonal if it wasn't consumed before
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };

        // Update the diagonal
        self.diagonal = newdiag;
    }
}

impl<S: scoring::Score> BestDirectionTracer for AllOptimal<S> {
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

        match self.diagonal.take() {
            None => {
                // Start the new path
                self.cache[row] = Some(partial::Path::new((row, col), score, self.is_roi));
            }
            Some(mut diagonal) => {
                // Extend & consume the diagonal
                diagonal.equivalent(row, col, score, self.is_roi);
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

impl<S: scoring::Score> GapTracer for AllOptimal<S> {
    type Score = S;

    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, score: S) {
        self.best_gap_row = match &self.cache[row - 1] {
            None => { unreachable!() }
            Some(x) => {
                let mut x = x.clone();
                x.gap(row, col, score);
                Some(x)
            }
        }
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, score: S) {
        match &mut self.best_gap_row {
            None => { unreachable!() }
            Some(x) => {
                x.gap(row, col, score);
            }
        }
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, score: S) {
        self.best_gap_col[row] = match &self.cache[row] {
            None => { unreachable!() }
            Some(x) => {
                let mut x = x.clone();
                x.gap(row, col, score);
                Some(x)
            }
        };
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, score: S) {
        match &mut self.best_gap_col[row] {
            None => { unreachable!() }
            Some(x) => {
                x.gap(row, col, score);
            }
        }
    }
}

impl<S: scoring::Score> Tracer for AllOptimal<S> {
    type Score = S;

    fn first_col_start(&mut self) {
        self.is_roi = self.rois.step(0);
    }
    fn first_col_end(&mut self) {
        self.diagonal = None;
    }

    fn col_start(&mut self, col: usize) {
        self.is_roi = self.rois.step(col);
    }
    fn col_end(&mut self, _: usize) {
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };
        self.diagonal = None;
        self.best_gap_row = None;
    }
}


impl<S: scoring::Score> Storage for AllOptimal<S> {
    fn reset(&mut self, newrows: usize, _: usize) {
        self.cache.clear();
        self.diagonal = None;
        self.best_gap_row = None;

        // TODO: reuse result vectors when possible
        self.results.clear();
        self.results.resize(newrows, Vec::new());

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
                    Some(x) => { self.save(x) }
                }
            };
            std::mem::swap(&mut cache, &mut self.cache);
        }


        self.results.iter().flatten().map(|x|
            AlignmentSeed { row: x.end.0, col: x.end.1, score: x.score }
        ).collect()
    }
}
