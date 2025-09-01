use std::borrow::Borrow;

use biobit_core_rs::num::PrimUInt;

use crate::Alignable;
use crate::pairwise::{Op, Step, scoring};

use super::step::StepWithOffset;

// pub fn prettify<L: PrimUInt + Into<usize>, Symbol: Display>(
//     seq1: &[Symbol],
//     seq2: &[Symbol],
//     steps: &[Step<L>],
// ) -> Result<[String; 3]> {
//     let total = steps.iter().map(|x| x.len.into()).sum::<usize>();
//     let mut lines = [
//         String::with_capacity(total + 1),
//         String::with_capacity(total + 1),
//         String::with_capacity(total + 1),
//     ];
//
//     let seq1_length = seq1.len();
//     let seq1_string = seq1.iter().map(|x| x.to_string()).collect::<String>();
//     if seq1_length != seq1_string.len() {
//         return Err(eyre::eyre!("seq1 string representation is non trivial"));
//     }
//     let mut seq1 = seq1_string.as_str();
//
//     let seq2_length = seq2.len();
//     let seq2_string = seq2.iter().map(|x| x.to_string()).collect::<String>();
//     if seq2_length != seq2_string.len() {
//         return Err(eyre::eyre!("seq2 string representation is non trivial"));
//     }
//     let mut seq2 = seq2_string.as_str();
//
//     for step in steps {
//         let len = step.len.to_usize().unwrap();
//
//         let symbol = match step.op {
//             Op::GapFirst | Op::GapSecond => " ",
//             Op::Equivalent => "~",
//             Op::Match => "|",
//             Op::Mismatch => "*",
//         }
//         .repeat(len);
//         lines[1].push_str(&symbol);
//
//         match step.op {
//             Op::GapFirst => {
//                 lines[0].push_str(&"-".repeat(len));
//                 lines[2].push_str(&seq2[..len]);
//
//                 seq2 = &seq2[len..];
//             }
//             Op::GapSecond => {
//                 lines[0].push_str(&seq1[len..]);
//                 lines[2].push_str(&"-".repeat(len));
//
//                 seq1 = &seq1[len..];
//             }
//             Op::Equivalent | Op::Mismatch | Op::Match => {
//                 lines[0].push_str(&seq1[len..]);
//                 lines[2].push_str(&seq2[len..]);
//
//                 seq1 = &seq1[len..];
//                 seq2 = &seq2[len..];
//             }
//         };
//     }
//     Ok(lines)
// }

pub fn disambiguate<Scheme, Seq1, Seq2>(
    ops: Vec<Step<u8>>,
    scoring: &Scheme,
    seq1: &Seq1,
    seq1offset: usize,
    seq2: &Seq2,
    seq2offset: usize,
) -> Vec<Step<u8>>
where
    Scheme: scoring::Scheme,
    Seq1: Alignable<Symbol = <Scheme as scoring::Scheme>::Symbol>,
    Seq2: Alignable<Symbol = <Scheme as scoring::Scheme>::Symbol>,
{
    let mut s1: usize = seq1offset;
    let mut s2: usize = seq2offset;
    let mut result = Vec::with_capacity(ops.len() * 2);
    for x in ops {
        match x.op() {
            Op::GapFirst => {
                s1 += *x.len() as usize;
                result.push(x);
            }
            Op::GapSecond => {
                s2 += *x.len() as usize;
                result.push(x);
            }
            Op::Equivalent => {
                let mut curop = scoring.classify(seq1.at(s1), seq2.at(s2));
                let mut len = 0;

                for _ in 0..*x.len() {
                    let op = scoring.classify(seq1.at(s1), seq2.at(s2));
                    if op == curop {
                        len += 1;
                    } else {
                        // Save results
                        let tail = len % (u8::MAX as usize);
                        if tail > 0 {
                            result.push(Step::new(curop.into(), tail as u8).unwrap());
                        }
                        for _ in 0..(len / (u8::MAX as usize)) {
                            result.push(Step::new(curop.into(), u8::MAX).unwrap());
                        }

                        curop = op;
                        len = 1;
                    }

                    s1 += 1;
                    s2 += 1;
                }
                // Save the last batch
                if len > 0 {
                    let tail = len % (u8::MAX as usize);
                    if tail > 0 {
                        result.push(Step::new(curop.into(), tail as u8).unwrap());
                    }
                    for _ in 0..(len / (u8::MAX as usize)) {
                        result.push(Step::new(curop.into(), u8::MAX).unwrap());
                    }
                }
            }
            Op::Match | Op::Mismatch => {
                s1 += *x.len() as usize;
                s2 += *x.len() as usize;
                result.push(x);
            }
        }
    }
    result
}

pub fn intersects<L, Seq1Idx, Seq2Idx, Accumulator>(
    mut iter1: impl Iterator<Item: Borrow<StepWithOffset<L, Seq1Idx, Seq2Idx>>>,
    mut iter2: impl Iterator<Item: Borrow<StepWithOffset<L, Seq1Idx, Seq2Idx>>>,
) -> bool
where
    L: PrimUInt + Into<Seq1Idx> + Into<Seq2Idx> + Into<Accumulator>,
    Seq1Idx: PrimUInt + Into<Accumulator>,
    Seq2Idx: PrimUInt + Into<Accumulator>,
    Accumulator: PrimUInt,
{
    let mut bbox_1 = match iter1.next() {
        None => return false,
        Some(x) => x,
    };
    let mut bbox_2 = match iter2.next() {
        None => return false,
        Some(x) => x,
    };

    // Main loop - tack seq1 coordinates
    loop {
        let borrow_bbox_1 = bbox_1.borrow();
        let borrow_bbox_2 = bbox_2.borrow();

        if borrow_bbox_1.intersects::<Accumulator>(borrow_bbox_2) {
            return true;
        }

        // If 2 alignments intersect - they must intersect by both projections
        // If they don't intersect by seq1 - they don't intersect at all
        match borrow_bbox_1.end().seq1.cmp(&borrow_bbox_2.end().seq1) {
            std::cmp::Ordering::Less => {
                bbox_1 = match iter1.next() {
                    None => return false,
                    Some(x) => x,
                };
            }
            std::cmp::Ordering::Greater => {
                bbox_2 = match iter2.next() {
                    None => return false,
                    Some(x) => x,
                };
            }
            std::cmp::Ordering::Equal => {
                bbox_1 = match iter1.next() {
                    None => return false,
                    Some(x) => x,
                };
                bbox_2 = match iter2.next() {
                    None => return false,
                    Some(x) => x,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // fn to_steps_with_offsets<'a>(
    //     steps: &'a [Step<u8>],
    //     offset: (u32, u32),
    // ) -> impl Iterator<Item = StepWithOffset<u8, u32, u32>> + 'a {
    //     StepsWithOffsetsIterator::new(steps.iter().cloned(), offset.into())
    // }
}
