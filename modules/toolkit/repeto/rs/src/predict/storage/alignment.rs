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
    pub score: S,
    // Whether the saved candidate passes the scan's fixed filter.
    valid: bool,

    // Running candidate alignment path
    candidate: EquivRunStats,
    run: EquivRun,
}

impl<S: scoring::Score> CandidatesTracker<S> {
    pub fn new<F: SoftFilter<Score = S>>(
        start: (usize, usize),
        score: S,
        roi_overlap: usize,
        filter: &F,
    ) -> Self {
        let run = EquivRun {
            in_roi: usize::from(roi_overlap > 0),
            length: 1,
        };
        let stats = EquivRunStats {
            in_roi: Length {
                max_len: run.in_roi,
                total_len: roi_overlap,
            },
            all: Length {
                max_len: 1,
                total_len: 1,
            },
        };

        Self {
            start,
            end: start,
            score,
            valid: filter.is_valid(&score, &stats),
            candidate: stats,
            run,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn gap<F: SoftFilter<Score = S>>(
        &mut self,
        row: usize,
        col: usize,
        newscore: S,
        filter: &F,
    ) {
        self.run = EquivRun::default();
        self.update(row, col, newscore, filter);
    }

    pub fn equivalent<F: SoftFilter<Score = S>>(
        &mut self,
        row: usize,
        col: usize,
        newscore: S,
        roi_overlap: usize,
        filter: &F,
    ) {
        self.run.length += 1;

        // Either arm can keep the ROI run going, so a simple > 0 check is sufficient.
        if roi_overlap > 0 {
            self.run.in_roi += 1;
        } else {
            self.run.in_roi = 0;
        }

        // Count equivalent positions and ROI bases in both arms.
        // ROI bases are counted twice: once for each arm. So roi_overlap is either 0, 1, or 2.
        //
        // These counts are the upper limit of matches, but they're useful for fast exclusion.
        // The final pass checks actual matches.
        self.candidate.all.total_len += 1;
        self.candidate.in_roi.total_len += roi_overlap;

        self.candidate.all.max_len = self.candidate.all.max_len.max(self.run.length);
        self.candidate.in_roi.max_len = self.candidate.in_roi.max_len.max(self.run.in_roi);

        self.update(row, col, newscore, filter);
    }

    fn update<F: SoftFilter<Score = S>>(
        &mut self,
        row: usize,
        col: usize,
        newscore: S,
        filter: &F,
    ) {
        // Statistics only grow. Once a candidate passes the minimum thresholds,
        // a higher score is enough to replace it without checking the filter again.
        let replace = if self.valid {
            newscore > self.score
        } else {
            filter.is_valid(&newscore, &self.candidate)
        };
        if replace {
            self.end = (row, col);
            self.score = newscore;
            self.valid = true;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_stops_after_the_first_valid_candidate() {
        use std::cell::Cell;

        struct CountingFilter(Cell<usize>);
        impl SoftFilter for CountingFilter {
            type Score = i32;

            fn is_valid(&self, score: &i32, stats: &EquivRunStats) -> bool {
                self.0.set(self.0.get() + 1);
                *score >= 2 && stats.all.total_len >= 2 && stats.all.max_len >= 2
            }
        }

        let filter = CountingFilter(Cell::new(0));
        let mut tracker = CandidatesTracker::new((0, 0), 3, 0, &filter);
        assert!(!tracker.is_valid());
        tracker.equivalent(1, 1, 2, 0, &filter);
        assert_eq!((tracker.end, tracker.score), ((1, 1), 2));
        assert!(tracker.is_valid());

        tracker.equivalent(2, 2, 1, 0, &filter);
        assert_eq!((tracker.end, tracker.score), ((1, 1), 2));
        assert_eq!(filter.0.get(), 2);

        tracker.equivalent(3, 3, 4, 0, &filter);
        tracker.gap(4, 3, 3, &filter);
        assert_eq!((tracker.end, tracker.score), ((3, 3), 4));
        assert!(tracker.is_valid());
        assert_eq!(filter.0.get(), 2);
    }
}
