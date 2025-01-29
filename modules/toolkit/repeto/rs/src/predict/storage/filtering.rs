pub trait SoftFilter {
    type Score;

    fn is_valid(&self, score: &Self::Score, stats: &EquivRunStats) -> bool;
}

#[derive(Debug, Eq, Copy, Clone, PartialEq, PartialOrd, Hash, Default)]
pub struct Length {
    /// Maximum length of an ungapped run
    pub max_len: usize,
    /// Total length of all ungapped runs
    pub total_len: usize,
}

#[derive(Debug, Eq, Copy, Clone, PartialEq, PartialOrd, Hash, Default)]
pub struct EquivRunStats {
    /// All ungapped diagonal (Equivalence) runs
    pub all: Length,
    /// All ungapped diagonal (Equivalence) runs intersecting requested ROIs
    pub in_roi: Length,
}
