use Option;

use biobit_alignment::pairwise::scoring;
use crate::

pub trait Filter {
    fn is_valid(&self, path: ) -> bool;
}


pub struct SpansThresholds {
    pub min_max_length: usize,
    pub min_max_roi_overlap: usize,
    pub min_max_roi_overlap_frac: f32,
    pub min_total_length: usize,
}

pub struct Filters<S: scoring::Score> {
    pub rois: Vec<(usize, usize)>,
    pub min_score: S,
    pub spans: SpansThresholds,
}


#[derive(Default)]
pub struct ThresholdsBuilder<S: scoring::Score> {
    rois: Vec<(usize, usize)>,

    min_score: Option<S>,
    spans_min_max_length: Option<usize>,
    spans_min_max_roi_overlap: Option<usize>,
    spans_min_max_roi_overlap_frac: Option<f32>,
    spans_min_total_length: Option<usize>,
}

impl<S: scoring::Score> ThresholdsBuilder<S> {
    pub fn new() -> Self { ThresholdsBuilder::default() }

    pub fn with_symbols_scoring(&mut self, complementary: Option<S>, mismatch: Option<S>) -> &mut Self {
        self.complementary = complementary;
        self.mismatch = mismatch;
        &mut self
    }

    pub fn with_gaps_scoring(&mut self, open: Option<S>, extend: Option<S>) -> &mut Self {
        self.gap_open = open;
        self.gap_extend = extend;
        &mut self
    }

    pub fn with_score(&mut self, min: Option<S>) -> &mut Self {
        self.min_score = min;
        &mut self
    }

    pub fn with_rois(&mut self, rois: Vec<(usize, usize)>) -> &mut Self {
        self.rois = rois;
        &mut self
    }

    pub fn with_spans(&mut self, min_max_length: Option<usize>, min_max_roi_overlap: Option<usize>, min_max_roi_overlap_frac: Option<f32>, min_total_length: Option<usize>) -> &mut Self {
        self.spans_min_max_length = min_max_length;
        self.spans_min_max_roi_overlap = min_max_roi_overlap;
        self.spans_min_max_roi_overlap_frac = min_max_roi_overlap_frac;
        self.spans_min_total_length = min_total_length;
        &mut self
    }

    pub fn build(self) -> Filters<S> {
        Filters {
            rois: self.rois,
            min_score: self.min_score.unwrap_or_default(),
            spans: SpansThresholds {
                min_max_length: self.spans_min_max_length.unwrap_or_default(),
                min_max_roi_overlap: self.spans_min_max_roi_overlap.unwrap_or_default(),
                min_max_roi_overlap_frac: self.spans_min_max_roi_overlap_frac.unwrap_or_default(),
                min_total_length: self.spans_min_total_length.unwrap_or_default(),
            },
        }
    }
}


