use std::iter::Peekable;

use super::scoring::symbols::EquivType;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AlignmentOp {
    GapFirst,
    GapSecond,

    // Equivalence = ambiguous, i.e. match OR mismatch
    // Might represent other meanings as well, i.e. similar AA in proteins
    Equivalent,
    Match,
    Mismatch,
}

impl AlignmentOp {
    pub fn symbol(&self) -> char {
        match self {
            AlignmentOp::GapFirst => 'v',
            AlignmentOp::GapSecond => '^',
            AlignmentOp::Equivalent => '~',
            AlignmentOp::Match => '=',
            AlignmentOp::Mismatch => 'X',
        }
    }
}

impl From<EquivType> for AlignmentOp {
    fn from(value: EquivType) -> Self {
        match value {
            EquivType::Match => AlignmentOp::Match,
            EquivType::Mismatch => AlignmentOp::Mismatch,
            EquivType::Equivalent => AlignmentOp::Equivalent,
        }
    }
}

// TODO: bitpack using 16 bit value?
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct AlignmentStep {
    pub op: AlignmentOp,
    pub len: u8,
}

#[derive(Copy, Clone)]
pub struct AlignmentOffset {
    pub seq1: usize,
    pub seq2: usize,
}


pub struct CoalescedAlignmentStep {
    pub start: AlignmentOffset,
    pub len: usize,
    pub op: AlignmentOp,
}

impl CoalescedAlignmentStep {
    pub fn end(&self) -> AlignmentOffset {
        let mut end = self.start;
        match self.op {
            AlignmentOp::GapFirst => { end.seq1 += self.len }
            AlignmentOp::GapSecond => { end.seq2 += self.len }
            AlignmentOp::Equivalent | AlignmentOp::Mismatch | AlignmentOp::Match => {
                end.seq1 += self.len;
                end.seq2 += self.len;
            }
        };
        end
    }
}

pub struct CoalescedAlignmentIter<'a, T: Iterator<Item=&'a AlignmentStep>> {
    pub iter: Peekable<T>,
    pub offset: AlignmentOffset,
}

impl<'a, T: Iterator<Item=&'a AlignmentStep>> Iterator for CoalescedAlignmentIter<'a, T> {
    type Item = CoalescedAlignmentStep;

    fn next(&mut self) -> Option<Self::Item> {
        let mut coalesced = match self.iter.next() {
            None => { return None; }
            Some(x) => CoalescedAlignmentStep {
                start: self.offset.clone(),
                len: x.len as usize,
                op: x.op,
            }
        };
        loop {
            match self.iter.peek() {
                Some(x) if x.op == coalesced.op => {
                    let x = self.iter.next().unwrap();
                    coalesced.len += x.len as usize;
                }
                _ => {
                    // Advance the offset
                    match coalesced.op {
                        AlignmentOp::GapFirst => { self.offset.seq1 += coalesced.len; }
                        AlignmentOp::GapSecond => { self.offset.seq2 += coalesced.len; }
                        AlignmentOp::Equivalent | AlignmentOp::Match | AlignmentOp::Mismatch => {
                            self.offset.seq1 += coalesced.len;
                            self.offset.seq2 += coalesced.len;
                        }
                    }
                    return Some(coalesced);
                }
            }
        }
    }
}


pub mod utils {
    use geo::{Intersects, Line};

    use super::super::alignment::CoalescedAlignmentStep;

    use super::{AlignmentOp, AlignmentStep};
    use super::super::Alignable;
    use super::super::scoring::ScoringScheme;

    pub fn disambiguate(ops: Vec<AlignmentStep>, scoring: &impl ScoringScheme,
                        seq1: &impl Alignable, seq1offset: usize,
                        seq2: &impl Alignable, seq2offset: usize) -> Vec<AlignmentStep> {
        let mut s1: usize = seq1offset;
        let mut s2: usize = seq2offset;
        let mut result = Vec::with_capacity(ops.len() * 2);
        for x in ops {
            match x.op {
                AlignmentOp::GapFirst => {
                    s1 += x.len as usize;
                    result.push(x);
                }
                AlignmentOp::GapSecond => {
                    s2 += x.len as usize;
                    result.push(x);
                }
                AlignmentOp::Equivalent => {
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
                                result.push(AlignmentStep { op: curop.into(), len: tail as u8 });
                            }
                            for _ in 0..(len / (u8::MAX as usize)) {
                                result.push(AlignmentStep { op: curop.into(), len: u8::MAX });
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
                            result.push(AlignmentStep { op: curop.into(), len: tail as u8 });
                        }
                        for _ in 0..(len / (u8::MAX as usize)) {
                            result.push(AlignmentStep { op: curop.into(), len: u8::MAX });
                        }
                    }
                }
                AlignmentOp::Match | AlignmentOp::Mismatch => {
                    s1 += x.len as usize;
                    s2 += x.len as usize;
                    result.push(x);
                }
            }
        };
        result
    }

    pub fn prettify(mut seq1: &str, mut seq2: &str, steps: &[AlignmentStep], total: usize) -> String {
        let mut lines = [
            String::with_capacity(total + 1),
            String::with_capacity(total + 1),
            String::with_capacity(total + 1)
        ];

        for step in steps {
            let len = step.len as usize;

            let symbol = match step.op {
                AlignmentOp::GapFirst | AlignmentOp::GapSecond => " ",
                AlignmentOp::Equivalent => "~",
                AlignmentOp::Match => "|",
                AlignmentOp::Mismatch => "*"
            }.repeat(len);
            lines[1].push_str(&symbol);

            match step.op {
                AlignmentOp::GapFirst => {
                    lines[0].push_str(&"-".repeat(len));
                    lines[2].push_str(&seq2[..len]);

                    seq2 = &seq2[len..];
                }
                AlignmentOp::GapSecond => {
                    lines[0].push_str(&seq1[len..]);
                    lines[2].push_str(&"-".repeat(len));

                    seq1 = &seq1[len..];
                }
                AlignmentOp::Equivalent | AlignmentOp::Mismatch | AlignmentOp::Match => {
                    lines[0].push_str(&seq1[len..]);
                    lines[2].push_str(&seq2[len..]);

                    seq1 = &seq1[len..];
                    seq2 = &seq2[len..];
                }
            };
        }

        return lines.into_iter().collect();
    }

    pub fn rle(steps: &[AlignmentStep], len: usize) -> String {
        // TODO collapse identical ops
        let mut result = String::with_capacity(len * 4 + 1);
        for step in steps {
            result.push_str(&step.len.to_string());
            result.push(step.op.symbol());
        }
        result
    }


    pub fn intersects(
        mut iter1: impl Iterator<Item=CoalescedAlignmentStep>,
        mut iter2: impl Iterator<Item=CoalescedAlignmentStep>,
    ) -> bool {
        fn toline(step: CoalescedAlignmentStep) -> Line<isize> {
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
}
