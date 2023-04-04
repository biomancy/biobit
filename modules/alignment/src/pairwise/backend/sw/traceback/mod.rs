use std::ops::Range;

pub use tracemat::TraceMatrix;

use super::Score;
use super::algo::{BestDirectionTracer, GapTracer, Tracer};
use super::super::super::{AlignmentStep};

mod tracemat;

pub struct TracedAlignment {
    pub ops: Vec<AlignmentStep>,
    pub seq1: Range<usize>,
    pub seq2: Range<usize>,
}

pub trait TraceMat: Tracer {
    fn reset(&mut self, rows: usize, cols: usize);
    fn trace(&self, row: usize, col: usize) -> Result<TracedAlignment, ()>;
}

// #[cfg(test)]
// pub mod test_suite {
//     use super::*;
//
//     struct Workload {
//         seed: (usize, usize),
//         seq1: (usize, usize),
//         seq2: (usize, usize),
//         ops: Vec<AlignmentStep>,
//     }
//
//     fn fill_tracer(tracer: &mut impl Tracer, rows: usize, cols: usize, traces: &[Trace]) {
//         debug_assert!(traces.len() == rows * cols);
//         tracer.reset(rows, cols);
//         for r in 0..rows {
//             for c in 0..cols {
//                 let ind = r * cols + c;
//                 match traces[ind] {
//                     Trace::None => tracer.restart(r, c),
//                     Trace::GapRow => tracer.gap_row(r, c),
//                     Trace::GapCol => tracer.gap_col(r, c),
//                     Trace::Equivalent => tracer.equivalent(r, c),
//                 }
//             }
//         }
//     }
//
//     fn ensure(tracer: &mut impl Tracer, w: Workload) {
//         let trace = tracer.trace(w.seed.0, w.seed.1).unwrap();
//         assert_eq!(trace.ops, w.ops);
//         assert_eq!(
//             trace.seq1range,
//             Range {
//                 start: w.seq1.0,
//                 end: w.seq1.1,
//             },
//             "Seq 1 ranges mismatch"
//         );
//         assert_eq!(
//             trace.seq2range,
//             Range {
//                 start: w.seq2.0,
//                 end: w.seq2.1,
//             },
//             "Seq 2 ranges mismatch"
//         );
//     }
//
//     pub fn run_all(tracer: &mut impl Tracer) {
//         outside_range(tracer);
//         simple(tracer);
//         long(tracer);
//         complex(tracer);
//     }
//
//     fn outside_range(tracer: &mut impl Tracer) {
//         for [(size_row, size_col), (trace_row, trace_col)] in [
//             [(0, 0), (0, 0)],
//             [(0, 0), (0, 1)],
//             [(11, 12), (11, 12)],
//             [(11, 12), (10, 13)],
//         ] {
//             tracer.reset(size_row, size_col);
//             assert!(tracer.trace(trace_row, trace_col).is_err())
//         }
//     }
//
//     fn simple(tracer: &mut impl Tracer) {
//         tracer.reset(4, 4);
//         tracer.equivalent(0, 0);
//         tracer.equivalent(1, 1);
//         tracer.gap_row(2, 1);
//         tracer.gap_col(2, 2);
//         tracer.equivalent(3, 3);
//
//         for (row, col) in [(0, 0), (1, 1), (2, 1), (2, 2), (3, 3)] {
//             assert!(tracer.trace(row, col).is_ok());
//         }
//         for (row, col) in [(0, 1), (4, 4), (3, 2), (1, 2)] {
//             assert!(tracer.trace(row, col).is_err());
//         }
//
//         ensure(
//             tracer,
//             Workload {
//                 seed: (3, 3),
//                 seq1: (0, 4),
//                 seq2: (0, 4),
//                 ops: vec![
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 2 },
//                     AlignmentStep { op: AlignmentOp::GapFirst, len: 1 },
//                     AlignmentStep { op: AlignmentOp::GapSecond, len: 1 },
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 1 },
//                 ],
//             },
//         )
//     }
//
//     fn long(tracer: &mut impl Tracer) {
//         tracer.reset(512, 512);
//         for i in 0..512 {
//             tracer.equivalent(i, i);
//         }
//         ensure(tracer, Workload {
//             seed: (511, 511),
//             seq1: (0, 512),
//             seq2: (0, 512),
//             ops: vec![
//                 AlignmentStep { op: AlignmentOp::Equivalent, len: 255 },
//                 AlignmentStep { op: AlignmentOp::Equivalent, len: 255 },
//                 AlignmentStep { op: AlignmentOp::Equivalent, len: 2 },
//             ],
//         })
//     }
//
//     fn complex(tracer: &mut impl Tracer) {
//         use Trace::*;
//         let traces = vec![
//             Equivalent, None, Equivalent, None, None, Equivalent, Equivalent, GapCol, None, Equivalent, GapCol, None,
//             None, GapRow, Equivalent, None, None, Equivalent, None, Equivalent, None, Equivalent, None, None,
//             GapCol, Equivalent, Equivalent, None, Equivalent, None, None, None, Equivalent, Equivalent, None,
//             Equivalent, GapCol, None, None, GapRow, GapCol, GapRow, None, GapRow, None, GapCol,
//             Equivalent, Equivalent,
//         ];
//         fill_tracer(tracer, 8, 6, &traces);
//         let workload = vec![
//             Workload {
//                 seed: (7, 1),
//                 seq1: (7, 8),
//                 seq2: (1, 1),
//                 ops: vec![AlignmentStep { op: AlignmentOp::GapFirst, len: 1 }],
//             },
//             Workload {
//                 seed: (6, 0),
//                 seq1: (6, 6),
//                 seq2: (0, 1),
//                 ops: vec![AlignmentStep { op: AlignmentOp::GapSecond, len: 1 }],
//             },
//             Workload {
//                 seed: (0, 0),
//                 seq1: (0, 1),
//                 seq2: (0, 1),
//                 ops: vec![AlignmentStep { op: AlignmentOp::Equivalent, len: 1 }],
//             },
//             Workload {
//                 seed: (0, 5),
//                 seq1: (0, 1),
//                 seq2: (5, 6),
//                 ops: vec![AlignmentStep { op: AlignmentOp::Equivalent, len: 1 }],
//             },
//             Workload {
//                 seed: (2, 5),
//                 seq1: (0, 3),
//                 seq2: (2, 6),
//                 ops: vec![
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 2 },
//                     AlignmentStep { op: AlignmentOp::GapSecond, len: 1 },
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 1 },
//                 ],
//             },
//             Workload {
//                 seed: (7, 5),
//                 seq1: (3, 8),
//                 seq2: (1, 6),
//                 ops: vec![
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 3 },
//                     AlignmentStep { op: AlignmentOp::GapFirst, len: 1 },
//                     AlignmentStep { op: AlignmentOp::GapSecond, len: 1 },
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 1 },
//                 ],
//             },
//             Workload {
//                 seed: (7, 4),
//                 seq1: (3, 8),
//                 seq2: (1, 5),
//                 ops: vec![
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 3 },
//                     AlignmentStep { op: AlignmentOp::GapFirst, len: 1 },
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 1 },
//                 ],
//             },
//             Workload {
//                 seed: (6, 5),
//                 seq1: (1, 7),
//                 seq2: (0, 6),
//                 ops: vec![
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 1 },
//                     AlignmentStep { op: AlignmentOp::GapSecond, len: 1 },
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 4 },
//                     AlignmentStep { op: AlignmentOp::GapFirst, len: 1 },
//                 ],
//             },
//             Workload {
//                 seed: (5, 2),
//                 seq1: (4, 6),
//                 seq2: (1, 3),
//                 ops: vec![AlignmentStep { op: AlignmentOp::Equivalent, len: 2 }],
//             },
//             Workload {
//                 seed: (2, 1),
//                 seq1: (1, 3),
//                 seq2: (0, 2),
//                 ops: vec![
//                     AlignmentStep { op: AlignmentOp::Equivalent, len: 1 },
//                     AlignmentStep { op: AlignmentOp::GapSecond, len: 1 },
//                     AlignmentStep { op: AlignmentOp::GapFirst, len: 1 },
//                 ],
//             },
//         ];
//         for w in workload {
//             ensure(tracer, w);
//         }
//     }
// }
