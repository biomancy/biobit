use geo::{Intersects, Line};

use crate::pairwise::alignment::{Op, Step, CoalescedStep};
use crate::Alignable;
use crate::pairwise::scoring;

pub fn disambiguate<Scheme, Seq1, Seq2>(
    ops: Vec<Step>, scoring: &Scheme,
    seq1: &Seq1, seq1offset: usize,
    seq2: &Seq2, seq2offset: usize,
) -> Vec<Step>
    where
        Scheme: scoring::Scheme,
        Seq1: Alignable<Symbol=<Scheme as scoring::Scheme>::Symbol>,
        Seq2: Alignable<Symbol=<Scheme as scoring::Scheme>::Symbol>,
{
    let mut s1: usize = seq1offset;
    let mut s2: usize = seq2offset;
    let mut result = Vec::with_capacity(ops.len() * 2);
    for x in ops {
        match x.op {
            Op::GapFirst => {
                s1 += x.len as usize;
                result.push(x);
            }
            Op::GapSecond => {
                s2 += x.len as usize;
                result.push(x);
            }
            Op::Equivalent => {
                let mut curop = scoring.classify(seq1.at(s1), seq2.at(s2));
                let mut len = 0;

                for _ in 0..x.len {
                    let op = scoring.classify(seq1.at(s1), seq2.at(s2));
                    if op == curop {
                        len += 1;
                    } else {
                        // Save results
                        let tail = len % (u8::MAX as usize);
                        if tail > 0 {
                            result.push(Step { op: curop.into(), len: tail as u8 });
                        }
                        for _ in 0..(len / (u8::MAX as usize)) {
                            result.push(Step { op: curop.into(), len: u8::MAX });
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
                        result.push(Step { op: curop.into(), len: tail as u8 });
                    }
                    for _ in 0..(len / (u8::MAX as usize)) {
                        result.push(Step { op: curop.into(), len: u8::MAX });
                    }
                }
            }
            Op::Match | Op::Mismatch => {
                s1 += x.len as usize;
                s2 += x.len as usize;
                result.push(x);
            }
        }
    };
    result
}

pub fn prettify(mut seq1: &str, mut seq2: &str, steps: &[Step], total: usize) -> String {
    let mut lines = [
        String::with_capacity(total + 1),
        String::with_capacity(total + 1),
        String::with_capacity(total + 1)
    ];

    for step in steps {
        let len = step.len as usize;

        let symbol = match step.op {
            Op::GapFirst | Op::GapSecond => " ",
            Op::Equivalent => "~",
            Op::Match => "|",
            Op::Mismatch => "*"
        }.repeat(len);
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

pub fn rle(steps: &[Step], len: usize) -> String {
    // TODO collapse identical ops
    let mut result = String::with_capacity(len * 4 + 1);
    for step in steps {
        result.push_str(&step.len.to_string());
        result.push(step.op.symbol());
    }
    result
}


pub fn intersects(
    mut iter1: impl Iterator<Item=CoalescedStep>,
    mut iter2: impl Iterator<Item=CoalescedStep>,
) -> bool {
    fn toline(step: CoalescedStep) -> Line<isize> {
        let end = step.end();
        Line::new(
            (step.start.seq2 as isize, step.start.seq1 as isize),
            (end.seq2 as isize - 1, end.seq1 as isize - 1),
        )
    }

    let mut first = match iter1.next() {
        None => { return false; }
        Some(x) => toline(x)
    };
    let mut second = match iter2.next() {
        None => { return false; }
        Some(x) => toline(x)
    };

    // TODO: optimize - fast forward X/Y where applicable

    // Detect overlaps
    loop {
        // debug_assert!(
        //     max(first.start.x, second.start.x) <= min(first.end.x, second.end.x)
        // );
        if first.intersects(&second) {
            return true;
        }

        if first.end.x < second.end.x {
            first = match iter1.next() {
                None => { return false; }
                Some(x) => { toline(x) }
            };
        } else if second.end.x < first.end.x {
            debug_assert!(second.end.x <= first.end.x);
            second = match iter2.next() {
                None => { return false; }
                Some(x) => { toline(x) }
            };
        } else {
            // Border situation - both segments end in the same position
            first = match iter1.next() {
                None => { return false; }
                Some(x) => { toline(x) }
            };
            second = match iter2.next() {
                None => { return false; }
                Some(x) => { toline(x) }
            };
        }
    }
}