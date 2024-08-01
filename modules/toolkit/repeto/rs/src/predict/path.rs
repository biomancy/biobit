use biobit_alignment::pairwise::scoring;

pub mod partial {

    use super::*;

    #[derive(Debug, Eq, Copy, Clone, PartialEq, Hash, Default)]
    pub struct Stat {
        pub current_length: usize,
        pub max_length: usize,
        pub cumulative_length: usize,
    }

    impl Stat {
        #[inline(always)]
        pub fn reset(&mut self) {
            if self.current_length != 0 {
                self.max_length = max(self.max_length, self.current_length);
                self.current_length = 0;
            }
        }

        #[inline(always)]
        pub fn extend(&mut self) {
            self.current_length += 1;
            self.cumulative_length += 1;
        }
    }

    #[derive(Debug, Eq, Copy, Clone, PartialEq, Hash, Default)]
    pub struct PathStat {
        pub roi: Stat,
        pub stem: Stat,
    }


    #[derive(Debug, Eq, Clone, PartialEq, Hash)]
    pub struct Path<S: scoring::Score> {
        pub start: (usize, usize),
        pub end: (usize, usize),
        pub optimal: PathStat,
        pub current: PathStat,
        pub score: S,
    }

    impl<S: scoring::Score> Path<S> {
        pub fn new(start: (usize, usize), score: S, is_roi: bool) -> Self {
            let mut stat = PathStat::default();
            stat.stem.extend();

            if is_roi {
                stat.roi.extend();
            }

            Self { start, end: start, current: stat, optimal: stat, score }
        }

        pub fn gap(&mut self, row: usize, col: usize, newscore: S) {
            self.current.roi.reset();
            self.current.stem.reset();
            self.update(row, col, newscore);
        }

        pub fn equivalent(&mut self, row: usize, col: usize, newscore: S, is_roi: bool) {
            self.current.stem.extend();
            if is_roi {
                self.current.roi.extend();
            } else {
                self.current.roi.reset();
            }

            self.update(row, col, newscore);
        }

        fn update(&mut self, row: usize, col: usize, newscore: S) {
            if newscore > self.score {
                self.end = (row, col);
                self.optimal = self.current;
                self.score = newscore;
            }

            debug_assert!(self.start.0 <= self.end.0);
            debug_assert!(self.start.1 <= self.end.1);
        }
    }

    impl<S: scoring::Score> Into<full::Path<S>> for Path<S> {
        fn into(self) -> full::Path<S> {
            full::Path {
                start: self.start,
                end: self.end,
                stat: full::PathStat {
                    roi: full::Stat {
                        max_length: self.optimal.roi.max_length,
                        cumulative_length: self.optimal.roi.cumulative_length,
                    },
                    stem: full::Stat {
                        max_length: self.optimal.stem.max_length,
                        cumulative_length: self.optimal.stem.cumulative_length,
                    },
                },
                score: self.score,
            }
        }
    }
}


pub mod full {
    use super::*;

    #[derive(Debug, Eq, Copy, Clone, PartialEq, Hash, Default)]
    pub struct Stat {
        pub max_length: usize,
        pub cumulative_length: usize,
    }

    #[derive(Debug, Eq, Copy, Clone, PartialEq, Hash, Default)]
    pub struct PathStat {
        pub roi: Stat,
        pub stem: Stat,
    }


    #[derive(Debug, Eq, Clone, PartialEq, Hash)]
    pub struct Path<S: scoring::Score> {
        pub start: (usize, usize),
        pub end: (usize, usize),
        pub stat: PathStat,
        pub score: S,
    }
}