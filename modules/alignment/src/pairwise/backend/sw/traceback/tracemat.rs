use std::ops::Range;

use super::{BestDirectionTracer, GapTracer, Score, TracedAlignment, TraceMat, Tracer};
use super::super::super::super::{AlignmentOp, AlignmentStep};

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

impl TryFrom<Trace> for AlignmentOp {
    type Error = ();

    fn try_from(value: Trace) -> Result<Self, Self::Error> {
        match value {
            Trace::Start => Err(()),
            Trace::GapRow => Ok(AlignmentOp::GapFirst),
            Trace::GapCol => Ok(AlignmentOp::GapSecond),
            Trace::Equivalent => Ok(AlignmentOp::Equivalent)
        }
    }
}


struct RunningTrace {
    pub op: AlignmentOp,
    pub len: usize,
}

impl RunningTrace {
    pub fn new(op: AlignmentOp, len: usize) -> Self {
        Self { op, len }
    }

    pub fn save(self, saveto: &mut Vec<AlignmentStep>) {
        let tail = self.len % (u8::MAX as usize);
        if tail > 0 {
            saveto.push(AlignmentStep { op: self.op, len: tail as u8 });
        }
        for _ in 0..(self.len / (u8::MAX as usize)) {
            saveto.push(AlignmentStep { op: self.op, len: u8::MAX });
        }
    }
}

pub struct TraceMatrix {
    best: Vec<Trace>,
    row_gap: Vec<GapTrace>,
    col_gap: Vec<GapTrace>,
    rows: usize,
    cols: usize,
}

impl TraceMatrix {
    pub fn new() -> Self {
        Self {
            best: Vec::new(),
            row_gap: Vec::new(),
            col_gap: Vec::new(),
            rows: 0,
            cols: 0,
        }
    }
}

impl Tracer for TraceMatrix {}

impl BestDirectionTracer for TraceMatrix {
    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize, _: Score) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::GapRow;
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize, _: Score) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::GapCol;
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize, _: Score) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::Equivalent;
    }

    #[inline(always)]
    fn none(&mut self, row: usize, col: usize) {
        self.best[(row + 1) * self.cols + (col + 1)] = Trace::Start;
    }
}

impl GapTracer for TraceMatrix {
    #[inline(always)]
    fn row_gap_open(&mut self, row: usize, col: usize, _: Score) {
        self.row_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Open;
    }

    #[inline(always)]
    fn row_gap_extend(&mut self, row: usize, col: usize, _: Score) {
        self.row_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Extend;
    }

    #[inline(always)]
    fn col_gap_open(&mut self, row: usize, col: usize, _: Score) {
        self.col_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Open;
    }

    #[inline(always)]
    fn col_gap_extend(&mut self, row: usize, col: usize, _: Score) {
        self.col_gap[(row + 1) * self.cols + (col + 1)] = GapTrace::Extend;
    }
}

impl TraceMat for TraceMatrix {
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

    fn trace(&self, row: usize, col: usize) -> Result<TracedAlignment, ()> {
        let (seq1end, seq2end) = (row + 1, col + 1);
        if seq1end >= self.rows || seq2end >= self.cols {
            return Err(());
        }

        let (mut row, mut col) = (seq1end, seq2end);
        let seed = match self.best[row * self.cols + col].try_into() {
            Err(()) => return Err(()),
            Ok(op) => op
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
                Ok(op) => op
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
        let mut seq1range = Range { start: row, end: seq1end };
        let mut seq2range = Range { start: col, end: seq2end };
        for x in [&mut seq1range, &mut seq2range] {
            if x.start == x.end {
                x.start -= 1;
                x.end -= 1;
            }
        }
        result.reverse();

        return Ok(TracedAlignment {
            ops: result,
            seq1: seq1range,
            seq2: seq2range,
        });
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use super::super::test_suite;
//
//     #[test]
//     fn test() {
//         let mut tracer = TraceMatrix::new();
//         test_suite::run_all(&mut tracer);
//     }
// }
