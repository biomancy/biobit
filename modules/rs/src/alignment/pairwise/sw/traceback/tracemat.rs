use std::marker::PhantomData;
use std::ops::Range;

use eyre::{eyre, Result};

use crate::alignment::pairwise::sw::algo::{BestDirectionTracer, GapTracer, Tracer};
use crate::alignment::pairwise::sw::traceback::{TraceMat, TracedAlignment};
use crate::alignment::pairwise::{scoring, Op, Step};

// TODO: implement to use only 2 bits
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Trace {
    Start,
    GapRow,
    GapCol,
    Equivalent,
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GapTrace {
    Open,
    Extend,
}

impl TryFrom<Trace> for Op {
    type Error = ();

    fn try_from(value: Trace) -> Result<Self, Self::Error> {
        match value {
            Trace::Start => Err(()),
            Trace::GapRow => Ok(Op::GapFirst),
            Trace::GapCol => Ok(Op::GapSecond),
            Trace::Equivalent => Ok(Op::Equivalent),
        }
    }
}

struct RunningTrace {
    pub op: Op,
    pub len: usize,
}

impl RunningTrace {
    pub fn new(op: Op, len: usize) -> Self {
        Self { op, len }
    }

    pub fn save(self, saveto: &mut Vec<Step>) {
        let tail = self.len % (u8::MAX as usize);
        if tail > 0 {
            saveto.push(Step {
                op: self.op,
                len: tail as u8,
            });
        }
        for _ in 0..(self.len / (u8::MAX as usize)) {
            saveto.push(Step {
                op: self.op,
                len: u8::MAX,
            });
        }
    }
}

pub struct TraceMatrix<S: scoring::Score> {
    best: Vec<Trace>,
    row_gap: Vec<GapTrace>,
    col_gap: Vec<GapTrace>,
    rows: usize,
    cols: usize,
    phantom: PhantomData<S>,
}

impl<S: scoring::Score> TraceMatrix<S> {
    pub fn new() -> Self {
        Self {
            best: Vec::new(),
            row_gap: Vec::new(),
            col_gap: Vec::new(),
            rows: 0,
            cols: 0,
            phantom: Default::default(),
        }
    }
}

impl<S: scoring::Score> Default for TraceMatrix<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: scoring::Score> Tracer for TraceMatrix<S> {
    type Score = S;
}

impl<S: scoring::Score> BestDirectionTracer for TraceMatrix<S> {
    type Score = S;

    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, _: Self::Score) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::GapRow;
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, _: Self::Score) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::GapCol;
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, _: Self::Score) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::Equivalent;
    }

    #[inline(always)]
    fn none(&mut self, row: usize, col: usize) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::Start;
    }
}

impl<S: scoring::Score> GapTracer for TraceMatrix<S> {
    type Score = S;

    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, _: Self::Score) {
        self.row_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Open;
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, _: Self::Score) {
        self.row_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Extend;
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, _: Self::Score) {
        self.col_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Open;
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, _: Self::Score) {
        self.col_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Extend;
    }
}

impl<S: scoring::Score> TraceMat for TraceMatrix<S> {
    fn reset(&mut self, rows: usize, cols: usize) {
        self.rows = rows + 1;
        self.cols = cols + 1;

        self.best.clear();
        self.best.resize(self.rows * self.cols, Trace::Start);

        self.row_gap.clear();
        self.row_gap.resize(self.rows * self.cols, GapTrace::Open);

        self.col_gap.clear();
        self.col_gap.resize(self.rows * self.cols, GapTrace::Open);
    }

    fn trace(&self, row: usize, col: usize) -> Result<TracedAlignment> {
        let (seq1end, seq2end) = (row + 1, col + 1);
        if seq1end >= self.rows || seq2end >= self.cols {
            return Err(eyre!("Out of bounds"));
        }

        let (mut row, mut col) = (seq1end, seq2end);
        let seed = match self.best[row * self.cols + col].try_into() {
            Err(()) => return Err(eyre!("Invalid seed")),
            Ok(op) => op,
        };

        let mut result = Vec::new();
        let mut trace = RunningTrace::new(seed, 0);

        loop {
            let op = self.best[row * self.cols + col];
            let aop = match op.try_into() {
                Err(()) => {
                    trace.save(&mut result);
                    break;
                }
                Ok(op) => op,
            };

            if aop == trace.op {
                trace.len += 1;
            } else {
                trace.save(&mut result);
                trace = RunningTrace::new(aop, 1);
            }

            match op {
                Trace::Start => {
                    debug_assert!(false, "Must be unreachable!");
                    break;
                }
                Trace::GapRow => {
                    while self.row_gap[row * self.cols + col] != GapTrace::Open {
                        trace.len += 1;
                        row -= 1;
                    }
                    row -= 1;
                }
                Trace::GapCol => {
                    while self.col_gap[row * self.cols + col] != GapTrace::Open {
                        trace.len += 1;
                        col -= 1;
                    }
                    col -= 1;
                }
                Trace::Equivalent => {
                    row -= 1;
                    col -= 1;
                }
            };
        }
        let mut seq1range = Range {
            start: row,
            end: seq1end,
        };
        let mut seq2range = Range {
            start: col,
            end: seq2end,
        };
        for x in [&mut seq1range, &mut seq2range] {
            if x.start == x.end {
                x.start -= 1;
                x.end -= 1;
            }
        }
        result.reverse();

        Ok(TracedAlignment {
            ops: result,
            seq1: seq1range,
            seq2: seq2range,
        })
    }
}
