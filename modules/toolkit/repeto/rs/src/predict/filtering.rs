use biobit_alignment_rs::pairwise::scoring;
use biobit_core_rs::loc::Segment;

use super::storage::filtering::{EquivRunStats, Length, SoftFilter};

#[derive(Debug, Eq, Clone, PartialEq, Hash, Default)]
pub struct Filter<S: scoring::Score> {
    min_score: S,
    stats: EquivRunStats,
    rois: Vec<Segment<usize>>,
}

impl<S: scoring::Score> Filter<S> {
    pub fn with_min_score(mut self, min_score: S) -> Self {
        self.min_score = min_score;
        self
    }

    pub fn with_rois(mut self, rois: Vec<Segment<usize>>) -> Self {
        self.rois = rois;
        self
    }

    pub fn with_min_roi_overlap(mut self, total: usize, ungapped: usize) -> Self {
        self.stats.in_roi = Length {
            total_len: total,
            max_len: ungapped,
        };
        self
    }

    pub fn with_min_matches(mut self, total: usize, ungapped: usize) -> Self {
        self.stats.all = Length {
            total_len: total,
            max_len: ungapped,
        };
        self
    }

    pub fn rois(&self) -> &[Segment<usize>] {
        &self.rois
    }
}

impl<S: scoring::Score> SoftFilter for Filter<S> {
    type Score = S;

    #[inline(always)]
    fn is_valid(&self, score: &Self::Score, stats: &EquivRunStats) -> bool {
        *score >= self.min_score
            && stats.in_roi.max_len >= self.stats.in_roi.max_len
            && stats.in_roi.total_len >= self.stats.in_roi.total_len
            && stats.all.max_len >= self.stats.all.max_len
            && stats.all.total_len >= self.stats.all.total_len
    }
}
