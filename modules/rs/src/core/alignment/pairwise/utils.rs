use super::op::Op;
use super::step::Step;

pub fn rle(steps: &[Step], len: usize) -> String {
    // TODO collapse identical ops
    let mut result = String::with_capacity(len * 4 + 1);
    for step in steps {
        result.push_str(&step.len.to_string());
        result.push(step.op.symbol());
    }
    result
}


pub fn prettify(mut seq1: &str, mut seq2: &str, steps: &[Step], total: usize) -> String {
    let mut lines = [
        String::with_capacity(total + 1),
        String::with_capacity(total + 1),
        String::with_capacity(total + 1),
    ];

    for step in steps {
        let len = step.len as usize;

        let symbol = match step.op {
            Op::GapFirst | Op::GapSecond => " ",
            Op::Equivalent => "~",
            Op::Match => "|",
            Op::Mismatch => "*",
        }
            .repeat(len);
        lines[1].push_str(&symbol);

        match step.op {
            Op::GapFirst => {
                lines[0].push_str(&"-".repeat(len));
                lines[2].push_str(&seq2[..len]);

                seq2 = &seq2[len..];
            }
            Op::GapSecond => {
                lines[0].push_str(&seq1[len..]);
                lines[2].push_str(&"-".repeat(len));

                seq1 = &seq1[len..];
            }
            Op::Equivalent | Op::Mismatch | Op::Match => {
                lines[0].push_str(&seq1[len..]);
                lines[2].push_str(&seq2[len..]);

                seq1 = &seq1[len..];
                seq2 = &seq2[len..];
            }
        };
    }

    lines.into_iter().collect()
}

// pub fn intersects(
//     mut iter1: impl Iterator<Item=CoalescedStep>,
//     mut iter2: impl Iterator<Item=CoalescedStep>,
// ) -> bool {
//     fn toline(step: CoalescedStep) -> Line<isize> {
//         let end = step.end();
//         Line::new(
//             (step.start.seq2 as isize, step.start.seq1 as isize),
//             (end.seq2 as isize - 1, end.seq1 as isize - 1),
//         )
//     }
//
//     let mut first = match iter1.next() {
//         None => {
//             return false;
//         }
//         Some(x) => toline(x),
//     };
//     let mut second = match iter2.next() {
//         None => {
//             return false;
//         }
//         Some(x) => toline(x),
//     };
//
//     // TODO: optimize - fast forward X/Y where applicable
//
//     // Detect overlaps
//     loop {
//         // debug_assert!(
//         //     max(first.start.x, second.start.x) <= min(first.end.x, second.end.x)
//         // );
//         if first.intersects(&second) {
//             return true;
//         }
//
//         match first.end.x.cmp(&second.end.x) {
//             Ordering::Less => {
//                 first = match iter1.next() {
//                     None => {
//                         return false;
//                     }
//                     Some(x) => toline(x),
//                 };
//             }
//             Ordering::Greater => {
//                 debug_assert!(second.end.x <= first.end.x);
//                 second = match iter2.next() {
//                     None => {
//                         return false;
//                     }
//                     Some(x) => toline(x),
//                 };
//             }
//             Ordering::Equal => {
//                 // Border situation - both segments end in the same position
//                 first = match iter1.next() {
//                     None => {
//                         return false;
//                     }
//                     Some(x) => toline(x),
//                 };
//                 second = match iter2.next() {
//                     None => {
//                         return false;
//                     }
//                     Some(x) => toline(x),
//                 };
//             }
//         }
//     }
// }
