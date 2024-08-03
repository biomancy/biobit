use biobit_alignment_rs::pairwise::scoring;

pub struct Scoring<S: scoring::Score> {
    pub complementary: S,
    pub mismatch: S,

    pub gap_open: S,
    pub gap_extend: S,
}
