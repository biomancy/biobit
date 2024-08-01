use biobit_alignment::pairwise::scoring;
use biobit_core_rs::alignment::pairwise::scoring;

pub struct Scoring<S: scoring::Score> {
    pub complementary: S,
    pub mismatch: S,

    pub gap_open: S,
    pub gap_close: S,
}
