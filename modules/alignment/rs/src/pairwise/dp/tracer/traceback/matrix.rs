use std::marker::PhantomData;
use std::ops::Range;

use biobit_bitpack::{Codec, Len, PackedArray};
use eyre::{Result, eyre};

use super::{Traceback, TracedAlignment};
use crate::pairwise::dp::{
    Cursor,
    layout::Layout,
    tracer::{BestTrace, CellScores, Trace, Tracer},
};
use crate::pairwise::{Op, Step, scoring};

type TracePair = PackedArray<Trace, Len<2>>;
const TRACES_PER_BYTE: usize = TracePair::LEN.get();

struct RunningTrace {
    pub op: Op,
    pub len: usize,
}

impl RunningTrace {
    pub fn new(op: Op, len: usize) -> Self {
        Self { op, len }
    }

    pub fn save(self, saveto: &mut Vec<Step<u8>>) {
        let tail = self.len % (u8::MAX as usize);
        if tail > 0 {
            saveto.push(Step::new(self.op, tail as u8).unwrap());
        }
        for _ in 0..(self.len / (u8::MAX as usize)) {
            saveto.push(Step::new(self.op, u8::MAX).unwrap());
        }
    }
}

/// Two traceback cells per byte, packed consecutively across column boundaries.
/// Traceback uses the scan layout to find predecessors in the packed buffer.
pub struct TraceMatrix<S: scoring::Score> {
    blocks: Vec<TracePair>,
    phantom: PhantomData<S>,
}

impl<S: scoring::Score> TraceMatrix<S> {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            phantom: PhantomData,
        }
    }

    #[inline]
    fn cell(&self, layout: &impl Layout, row: usize, col: usize) -> Trace {
        if row == 0 || col == 0 {
            return Trace::Start;
        }
        let Some(index) = layout.get_index(col - 1, row - 1) else {
            return Trace::Start;
        };
        self.at(index)
    }

    #[inline]
    fn at(&self, index: usize) -> Trace {
        let pair = &self.blocks[index / TRACES_PER_BYTE];
        Trace::decode(if index.is_multiple_of(TRACES_PER_BYTE) {
            pair.first()
        } else {
            pair.last()
        })
    }

    #[inline]
    fn write(&mut self, index: usize, traces: &[Trace]) {
        if traces.is_empty() {
            return;
        }
        let dest = &mut self.blocks[index / TRACES_PER_BYTE..];
        let head = index % TRACES_PER_BYTE;
        // An odd start fills the low bits of a byte shared with the preceding cell.
        if head != 0 {
            let _ = dest[0].set(head, Trace::encode(&traces[0]));
        }
        let (pairs, tail) = traces[head..].as_chunks::<TRACES_PER_BYTE>();
        for (block, [first, second]) in dest[head..].iter_mut().zip(pairs) {
            *block = TracePair::from_array(&[Trace::encode(first), Trace::encode(second)]);
        }
        // A final unpaired trace fills the high bits and preserves the next cell.
        if let [last] = tail {
            let _ = dest[head + pairs.len()].set(0, Trace::encode(last));
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

    fn initialize(&mut self, layout: &impl Layout) {
        self.blocks.clear();
        let empty = TracePair::from_array(&[Trace::encode(&Trace::Start); TRACES_PER_BYTE]);
        self.blocks
            .resize(layout.len().div_ceil(TRACES_PER_BYTE), empty);
    }

    #[inline]
    fn chunk<const N: usize>(
        &mut self,
        _: &impl Layout,
        pointer: &Cursor,
        _: &[CellScores<S>; N],
        traces: &[Trace; N],
    ) {
        self.write(pointer.index, traces);
    }

    #[inline]
    fn tail(&mut self, _: &impl Layout, pointer: &Cursor, _: &[CellScores<S>], traces: &[Trace]) {
        self.write(pointer.index, traces);
    }
}

impl<S: scoring::Score> Traceback for TraceMatrix<S> {
    fn trace(&self, layout: &impl Layout, col: usize, row: usize) -> Result<TracedAlignment> {
        if self.blocks.len() != layout.len().div_ceil(TRACES_PER_BYTE) {
            return Err(eyre!("Layout size differs from the last scan"));
        }
        let index = layout
            .get_index(col, row)
            .ok_or_else(|| eyre!("Cell outside the scanned region"))?;
        let mut cell = self.at(index);
        let seed = cell.best().try_into().map_err(|()| eyre!("Invalid seed"))?;
        let (seq1end, seq2end) = (row + 1, col + 1);
        let (mut row, mut col) = (seq1end, seq2end);

        let mut result = Vec::new();
        let mut trace = RunningTrace::new(seed, 0);

        loop {
            let op = cell.best();
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
                BestTrace::Start => {
                    debug_assert!(false, "Must be unreachable!");
                    break;
                }
                BestTrace::GapRow => {
                    while cell.row_gap_extends() {
                        trace.len += 1;
                        row -= 1;
                        cell = self.cell(layout, row, col);
                    }
                    row -= 1;
                }
                BestTrace::GapCol => {
                    while cell.col_gap_extends() {
                        trace.len += 1;
                        col -= 1;
                        cell = self.cell(layout, row, col);
                    }
                    col -= 1;
                }
                BestTrace::Equivalent => {
                    row -= 1;
                    col -= 1;
                }
            }
            cell = self.cell(layout, row, col);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_chunk_boundaries_preserve_adjacent_trace_cells() {
        let mut matrix = TraceMatrix::<i32>::new();
        let layout = crate::pairwise::dp::layout::FullMatrix::new(7, 2).unwrap();
        matrix.initialize(&layout);

        for col in 0..2 {
            for rows in [4..7, 1..4, 0..1] {
                let traces: Vec<_> = rows
                    .clone()
                    .map(|row| match (col, row % 2) {
                        (0, 0) => Trace::EquivalentExtendRow,
                        (1, 1) => Trace::EquivalentExtendCol,
                        _ => Trace::Equivalent,
                    })
                    .collect();
                let scores = vec![
                    CellScores {
                        best: 2,
                        row_gap: 1,
                        col_gap: 1
                    };
                    traces.len()
                ];
                let pointer = Cursor {
                    col,
                    row: rows.start,
                    index: col * 7 + rows.start,
                };
                matrix.tail(&layout, &pointer, &scores, &traces);
            }
        }

        for col in 0..2 {
            for row in 0..7 {
                let cell = matrix.cell(&layout, row + 1, col + 1);
                assert_eq!(cell.best(), BestTrace::Equivalent);
                assert_eq!(cell.row_gap_extends(), col == 0 && row % 2 == 0);
                assert_eq!(cell.col_gap_extends(), col == 1 && row % 2 != 0);
            }
        }

        assert_eq!(matrix.blocks.len(), 7);
        let layout = crate::pairwise::dp::layout::AboveAntiDiagonal::new(7, 2, 5).unwrap();
        matrix.initialize(&layout);

        assert_eq!(matrix.blocks.len(), 2);
        assert!(matrix.trace(&layout, 1, 0).is_err()); // cleared seed inside the triangle
        assert!(matrix.trace(&layout, 1, 1).is_err());
        assert!(matrix.trace(&layout, 2, 1).is_err());
    }
}
