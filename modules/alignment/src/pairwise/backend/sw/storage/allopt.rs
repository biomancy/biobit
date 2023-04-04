use super::{AlignmentSeed, Score, Storage, BestDirectionTracer, GapTracer, Tracer};

// An efficient algorithm to locate all locally optimal alignments between two sequences allowing for gaps
// 10.1093/bioinformatics/9.6.729


#[derive(Debug, Eq, Clone, PartialEq, Hash)]
struct Path {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub score: Score,
}

impl Path {
    fn new(start: (usize, usize), score: Score) -> Self {
        Self { start, end: start, score }
    }

    fn extend(&mut self, row: usize, col: usize, newscore: Score) {
        // debug_assert!(self.score < newscore);
        if newscore > self.score {
            self.end = (row, col);
            self.score = newscore;
        }

        debug_assert!(self.start.0 <= self.end.0);
        debug_assert!(self.start.1 <= self.end.1);
    }
}

pub struct AllOptimal {
    // Thresholds
    pub minscore: Score,

    // Main cache
    diagonal: Option<Path>,
    savediag: bool,
    cache: Vec<Option<Path>>,

    // Gapped paths caches
    best_gap_row: Option<Path>,
    best_gap_col: Vec<Option<Path>>,

    // Cache for finished paths in each row
    results: Vec<Vec<Path>>,
}

impl AllOptimal {
    pub fn new() -> Self {
        Self {
            minscore: 0,
            diagonal: None,
            savediag: false,
            cache: vec![None; 128],
            best_gap_row: None,
            best_gap_col: vec![None; 128],
            results: vec![Vec::new(); 128],
        }
    }

    fn save(&mut self, p: Path) {
        if p.score < self.minscore {
            return;
        }

        let row = p.start.0;
        for r in &mut self.results[row] {
            if r.start == p.start {
                // If match & better score -> update the hit
                if r.score < p.score {
                    *r = p;
                }
                return;
            }
        }
        // New match -> store the new path
        self.results[row].push(p)
    }

    #[inline(always)]
    fn update_diagonal(&mut self, newdiag: Option<Path>) {
        // Try to save the previous diagonal if it wasn't consumed before
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };

        // Update the diagonal
        self.diagonal = newdiag;
    }
}

impl BestDirectionTracer for AllOptimal {
    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, score: Score) {
        let newdiag = self.cache[row].take();

        // debug_assert!(self.best_gap_row.as_ref().unwrap().end == (row, col));
        // debug_assert!(self.best_gap_row.as_ref().unwrap().score == score);
        self.cache[row] = self.best_gap_row.clone();

        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, score: Score) {
        let newdiag = self.cache[row].take();

        // debug_assert!(self.best_gap_col[row].as_ref().unwrap().end == (row, col));
        // debug_assert!(self.best_gap_col[row].as_ref().unwrap().score == score);
        self.cache[row] = self.best_gap_col[row].clone();

        self.update_diagonal(newdiag);
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, score: Score) {
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

impl GapTracer for AllOptimal {
    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, score: Score) {
        self.best_gap_row = match &self.cache[row - 1] {
            None => { unreachable!() }
            Some(x) => {
                let mut x = x.clone();
                x.extend(row, col, score);
                Some(x)
            }
        }
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, score: Score) {
        match &mut self.best_gap_row {
            None => { unreachable!() }
            Some(x) => {
                x.extend(row, col, score);
            }
        }
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, score: Score) {
        self.best_gap_col[row] = match &self.cache[row] {
            None => { unreachable!() }
            Some(x) => {
                let mut x = x.clone();
                x.extend(row, col, score);
                Some(x)
            }
        };
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, score: Score) {
        match &mut self.best_gap_col[row] {
            None => { unreachable!() }
            Some(x) => {
                x.extend(row, col, score);
            }
        }
    }
}

impl Tracer for AllOptimal {
    #[inline(always)]
    fn first_col_end(&mut self) {
        self.diagonal = None;
    }

    #[inline(always)]
    fn col_end(&mut self, col: usize) {
        if let Some(diagonal) = self.diagonal.take() {
            self.save(diagonal);
        };
        self.diagonal = None;
        self.best_gap_row = None;
    }
}


impl Storage for AllOptimal {
    fn reset(&mut self, newrows: usize, _: usize) {
        self.cache.clear();
        self.diagonal = None;
        self.savediag = false;

        // TODO: reuse result vectors when possible
        self.results.clear();
        self.results.resize(newrows, Vec::new());

        self.cache.clear();
        self.cache.resize(newrows, None);
    }

    fn finalize(&mut self) -> Vec<AlignmentSeed> {
        {
            let mut cache = Vec::new();
            std::mem::swap(&mut cache, &mut self.cache);

            for x in &mut cache {
                match x.take() {
                    None => {}
                    Some(x) => { self.save(x) }
                }
            };
            std::mem::swap(&mut cache, &mut self.cache);
        }


        self.results.iter().flatten().map(|x|
            AlignmentSeed { row: x.end.0, col: x.end.1, score: x.score }
        ).collect()
    }
}
