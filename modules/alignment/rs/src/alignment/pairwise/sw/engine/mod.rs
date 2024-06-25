use tracers::Tracers;

use crate::analysis::alignment::pairwise::sw::{algo, storage, traceback};
use crate::analysis::alignment::pairwise::{alignment, scoring};
use crate::analysis::alignment::Alignable;

mod tracers;

pub struct Engine<S, Smb, Storage, TraceMat, Scheme>
where
    S: scoring::Score,
    Scheme: scoring::Scheme<Score = S, Symbol = Smb>,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    algo: algo::FullScan<S>,
    scoring: Scheme,
    tracers: Tracers<S, Storage, TraceMat>,
}

impl<S, Smb, Storage, TraceMat, Scheme> Engine<S, Smb, Storage, TraceMat, Scheme>
where
    S: scoring::Score,
    Scheme: scoring::Scheme<Score = S, Symbol = Smb>,
    Storage: storage::Storage + algo::Tracer<Score = S>,
    TraceMat: traceback::TraceMat + algo::Tracer<Score = S>,
{
    pub fn new(storage: Storage, tracemat: TraceMat, scoring: Scheme) -> Self {
        let algo = algo::FullScan::new(0);
        let tracers = Tracers {
            storage,
            tracemat,
            _phantom: Default::default(),
        };
        Self {
            algo,
            scoring,
            tracers,
        }
    }

    pub fn with_scoring(&mut self, scoring: Scheme) {
        self.scoring = scoring;
    }

    pub fn storage(&mut self) -> &mut Storage {
        &mut self.tracers.storage
    }

    pub fn scan_all<S1, S2>(&mut self, seq1: &S1, seq2: &S2) -> Vec<alignment::Alignment<S>>
    where
        S1: Alignable<Symbol = Smb>,
        S2: Alignable<Symbol = Smb>,
    {
        if seq1.len() == 0 || seq2.len() == 0 {
            return vec![];
        }

        self.tracers.reset(seq1.len(), seq2.len());
        self.algo
            .scan_all(seq1, seq2, &mut self.scoring, &mut self.tracers);
        self._finalize(seq1, seq2)
    }
    pub fn scan_up_triangle<S1, S2>(
        &mut self,
        seq1: &S1,
        seq2: &S2,
        offset: usize,
    ) -> Vec<alignment::Alignment<S>>
    where
        S1: Alignable<Symbol = Smb>,
        S2: Alignable<Symbol = Smb>,
    {
        if seq1.len() == 0 || seq2.len() == 0 {
            return vec![];
        }

        self.tracers.reset(seq1.len(), seq2.len());
        self.algo
            .scan_up_triangle(seq1, seq2, offset, &mut self.scoring, &mut self.tracers);
        self._finalize(seq1, seq2)
    }

    fn _finalize<S1, S2>(&mut self, seq1: &S1, seq2: &S2) -> Vec<alignment::Alignment<S>>
    where
        S1: Alignable<Symbol = Smb>,
        S2: Alignable<Symbol = Smb>,
    {
        self.tracers
            .storage
            .finalize()
            .into_iter()
            .map(|x| {
                let trace = self.tracers.tracemat.trace(x.row, x.col).unwrap();
                debug_assert_eq!(trace.seq1.end, x.row + 1);
                debug_assert_eq!(trace.seq2.end, x.col + 1);
                let ops = alignment::utils::disambiguate(
                    trace.ops,
                    &self.scoring,
                    seq1,
                    trace.seq1.start,
                    seq2,
                    trace.seq2.start,
                );

                alignment::Alignment {
                    score: x.score,
                    steps: ops,
                    seq1: trace.seq1,
                    seq2: trace.seq2,
                }
            })
            .collect()
        // Collapse overlapping paths & save results
        // let result = Vec::with_capacity(128);
        // for item in results.into_iter().rev() {
        //     let intersects = saveto[startind..].iter().any(|x| x.intersects(&item));
        //     if !intersects {
        //         saveto.push(item);
        //     }
        // }
    }
}
