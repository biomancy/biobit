use derive_getters::{Dissolve, Getters};

use biobit_alignment_rs::pairwise::scoring;
use biobit_core_rs::loc::Interval;

use super::storage::filtering::{EquivRunStats, Length, SoftFilter};

#[derive(Clone, PartialEq, PartialOrd, Debug, Hash, Default, Dissolve, Getters)]
pub struct Filter<S: scoring::Score> {
    min_score: S,
    stats: EquivRunStats,
    rois: Vec<Interval<usize>>,
}

impl<S: scoring::Score> Filter<S> {
    pub fn set_min_score(&mut self, min_score: S) -> &mut Self {
        self.min_score = min_score;
        self
    }

    pub fn set_rois(&mut self, mut rois: Vec<Interval<usize>>) -> &mut Self {
        self.rois = Interval::merge(&mut rois);
        self
    }

    pub fn set_min_roi_overlap(&mut self, total: usize, ungapped: usize) -> &mut Self {
        self.stats.in_roi = Length {
            total_len: total,
            max_len: ungapped,
        };
        self
    }

    pub fn set_min_matches(&mut self, total: usize, ungapped: usize) -> &mut Self {
        self.stats.all = Length {
            total_len: total,
            max_len: ungapped,
        };
        self
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
