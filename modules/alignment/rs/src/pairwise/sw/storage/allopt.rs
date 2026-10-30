use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::pairwise::scoring;
use crate::pairwise::sw::algo::{BestOrientationTracer, GapTracer, Tracer};
use crate::pairwise::sw::storage::{AlignmentSeed, Storage};

// An efficient algorithm to locate all locally optimal alignments between two sequences allowing for gaps
// 10.1093/bioinformatics/9.6.729

#[derive(Debug, Eq, Clone, PartialEq, Hash)]
struct Path<S: scoring::Score> {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub score: S,
}

impl<S: scoring::Score> Path<S> {
    fn new(start: (usize, usize), score: S) -> Self {
        Self {
            start,
            end: start,
            score,
        }
    }

    fn extend(&mut self, row: usize, col: usize, newscore: S) {
        // debug_assert!(self.score < newscore);
        if newscore > self.score {
            self.end = (row, col);
            self.score = newscore;
        }

        debug_assert!(self.start.0 <= self.end.0);
        debug_assert!(self.start.1 <= self.end.1);
    }
}

pub struct AllOptimal<S: scoring::Score> {
    // Thresholds
    pub minscore: S,

    // Main cache
    diagonal: Option<Path<S>>,
    cache: Vec<Option<Path<S>>>,

    // Gapped paths caches
    best_gap_row: Option<Path<S>>,
    best_gap_col: Vec<Option<Path<S>>>,

    // Cache for finished paths in each row
    results: Vec<Vec<Path<S>>>,
    // Starting column -> result index, preserving the order of first insertion.
    result_indices: Vec<HashMap<usize, usize>>,
}

impl<S: scoring::Score> AllOptimal<S> {
    pub fn new(minscore: S) -> Self {
        Self {
            minscore,
            diagonal: None,
            cache: vec![None; 128],
            best_gap_row: None,
            best_gap_col: vec![None; 128],
            results: vec![Vec::new(); 128],
            result_indices: vec![HashMap::new(); 128],
        }
    }

    fn save(&mut self, p: Path<S>) {
        if p.score < self.minscore {
            return;
        }

        let row = p.start.0;
        match self.result_indices[row].entry(p.start.1) {
            Entry::Occupied(entry) => {
                let r = &mut self.results[row][*entry.get()];
                if r.score < p.score {
                    *r = p;
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(self.results[row].len());
                self.results[row].push(p);
            }
        }
    }

    #[inline(always)]
    fn update_diagonal(&mut self, newdiag: Option<Path<S>>) {
        // Try to save the previous diagonal if it wasn't consumed before
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };

        // Update the diagonal
        self.diagonal = newdiag;
    }
}

impl<S: scoring::Score> BestOrientationTracer for AllOptimal<S> {
    type Score = S;

    #[inline(always)]
    fn gap_row(&mut self, row: usize, _: usize, _: S) {
        let newdiag = self.cache[row].take();

        // debug_assert!(self.best_gap_row.as_ref().unwrap().end == (row, col));
        // debug_assert!(self.best_gap_row.as_ref().unwrap().score == score);
        self.cache[row].clone_from(&self.best_gap_row);
        // self.cache[row] = self.best_gap_row.clone();

        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, _: usize, _: S) {
        let newdiag = self.cache[row].take();

        // debug_assert!(self.best_gap_col[row].as_ref().unwrap().end == (row, col));
        // debug_assert!(self.best_gap_col[row].as_ref().unwrap().score == score);
        // self.cache[row] = self.best_gap_col[row].clone();
        self.cache[row].clone_from(&self.best_gap_col[row]);

        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: S) {
        let newdiag = self.cache[row].take();

        match self.diagonal.take() {
            None => {
                // Start the new path
                self.cache[row] = Some(Path::new((row, col), score));
            }
            Some(mut diagonal) => {
                // Extend & consume the diagonal
                diagonal.extend(row, col, score);
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
            None => {
                unreachable!()
            }
            Some(x) => {
                let mut x = x.clone();
                x.extend(row, col, score);
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
                x.extend(row, col, score);
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
                x.extend(row, col, score);
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
                x.extend(row, col, score);
            }
        }
    }
}

impl<S: scoring::Score> Tracer for AllOptimal<S> {
    type Score = S;

    #[inline(always)]
    fn first_col_end(&mut self) {
        self.diagonal = None;
    }

    #[inline(always)]
    fn col_end(&mut self, _: usize) {
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };
        self.diagonal = None;
        self.best_gap_row = None;
    }
}

impl<S: scoring::Score> Storage for AllOptimal<S> {
    fn reset(&mut self, (_, rows): (usize, usize), _: (usize, usize)) {
        self.diagonal = None;
        self.best_gap_row = None;

        self.results.resize_with(rows, Vec::new);
        for results in &mut self.results {
            results.clear();
        }
        self.result_indices.resize_with(rows, HashMap::new);
        for indices in &mut self.result_indices {
            indices.clear();
        }

        self.cache.clear();
        self.cache.resize(rows, None);

        self.best_gap_col.clear();
        self.best_gap_col.resize(rows, None);
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
            .flatten()
            .map(|x| AlignmentSeed {
                row: x.end.0,
                col: x.end.1,
                score: x.score,
            })
            .collect()
    }
}
