use biobit_alignment_rs::pairwise::scoring;

use super::filtering::{EquivRunStats, Length, SoftFilter};

#[derive(Debug, Eq, Copy, Clone, PartialEq, Hash, Default)]
struct EquivRun {
    /// Current length of the overlap with ROIs
    pub in_roi: usize,
    /// Total length of the run
    pub length: usize,
}

#[derive(Debug, Eq, Clone, PartialEq, Hash)]
pub struct CandidatesTracker<S: scoring::Score> {
    // Current optimal alignment path
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub stats: EquivRunStats,
    pub score: S,
    // Running candidate alignment path
    candidate: EquivRunStats,
    run: EquivRun,
}

impl<S: scoring::Score> CandidatesTracker<S> {
    pub fn new(start: (usize, usize), score: S, is_roi: bool) -> Self {
        let run = EquivRun {
            in_roi: is_roi as usize,
            length: 1,
        };
        let stats = EquivRunStats {
            in_roi: Length {
                max_len: is_roi as usize,
                total_len: is_roi as usize,
            },
            all: Length {
                max_len: 1,
                total_len: 1,
            },
        };

        Self {
            start,
            end: start,
            stats,
            score,
            candidate: stats,
            run,
        }
    }

    pub fn gap<F: SoftFilter<Score = S>>(
        &mut self,
        row: usize,
        col: usize,
        newscore: S,
        filter: &F,
    ) {
        // When a gap occurs in the alignment we need to flush the current length of the run
        // And reset the run to zeros
        if self.run.length > self.candidate.all.max_len {
            self.candidate.all.max_len = self.run.length;
        }
        self.run.length = 0;

        if self.run.in_roi > self.candidate.in_roi.max_len {
            self.candidate.in_roi.max_len = self.run.in_roi;
        }
        self.run.in_roi = 0;

        // Try to update the optimal alignment
        self.update(row, col, newscore, filter);
    }

    pub fn equivalent<F: SoftFilter<Score = S>>(
        &mut self,
        row: usize,
        col: usize,
        newscore: S,
        is_roi: bool,
        filter: &F,
    ) {
        // Increment length for the equivalence stats
        self.candidate.all.total_len += 1;
        self.run.length += 1;

        // Increment length for the ROI if it's an overlap, reset the counter otherwise
        if is_roi {
            self.candidate.in_roi.total_len += 1;
            self.run.in_roi += 1;
        } else {
            self.candidate.in_roi.max_len = self.candidate.in_roi.max_len.max(self.run.in_roi);
            self.run.in_roi = 0;
        }

        self.update(row, col, newscore, filter);
    }

    fn update<F: SoftFilter<Score = S>>(
        &mut self,
        row: usize,
        col: usize,
        newscore: S,
        filter: &F,
    ) {
        // Try updating the optimal alignment if and only if the new score is better
        // and the candidate is passing the filter
        if newscore > self.score && filter.is_valid(&newscore, &self.candidate) {
            self.end = (row, col);
            self.stats = self.candidate;
            self.score = newscore;
        }

        debug_assert!(self.start.0 <= self.end.0);
        debug_assert!(self.start.1 <= self.end.1);
    }
}

#[derive(Debug, Eq, Clone, PartialEq, Hash)]
pub struct MayBeLocallyOptimal<S: scoring::Score> {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub score: S,
}

impl<S: scoring::Score> From<CandidatesTracker<S>> for MayBeLocallyOptimal<S> {
    fn from(value: CandidatesTracker<S>) -> Self {
        Self {
            start: value.start,
            end: value.end,
            score: value.score,
        }
    }
}
