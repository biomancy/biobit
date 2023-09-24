use std::cmp::{max, min};
use std::iter::Peekable;
use std::ops::Range;

use crate::pairwise::scoring;

pub mod utils;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Op {
    GapFirst,
    GapSecond,
    // Equivalence = ambiguous, i.e. match OR mismatch
    // Might represent other meanings as well, i.e. similar amino acids in proteins
    Equivalent,
    Match,
    Mismatch,
}

impl Op {
    pub fn symbol(&self) -> char {
        match self {
            Op::GapFirst => 'v',
            Op::GapSecond => '^',
            Op::Equivalent => '~',
            Op::Match => '=',
            Op::Mismatch => 'X',
        }
    }
}

impl From<scoring::equiv::Type> for Op {
    fn from(value: scoring::equiv::Type) -> Self {
        match value {
            scoring::equiv::Type::Match => Op::Match,
            scoring::equiv::Type::Mismatch => Op::Mismatch,
            scoring::equiv::Type::Equivalent => Op::Equivalent,
        }
    }
}

// TODO: bitpack using 16 bit value?
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Step {
    pub op: Op,
    pub len: u8,
}

#[derive(Copy, Clone)]
pub struct Offset {
    pub seq1: usize,
    pub seq2: usize,
}


pub struct CoalescedStep {
    pub start: Offset,
    pub len: usize,
    pub op: Op,
}

impl CoalescedStep {
    pub fn end(&self) -> Offset {
        let mut end = self.start;
        match self.op {
            Op::GapFirst => { end.seq1 += self.len }
            Op::GapSecond => { end.seq2 += self.len }
            Op::Equivalent | Op::Mismatch | Op::Match => {
                end.seq1 += self.len;
                end.seq2 += self.len;
            }
        };
        end
    }
}

pub struct CoalescedStepIter<'a, T: Iterator<Item=&'a Step>> {
    pub iter: Peekable<T>,
    pub offset: Offset,
}

impl<'a, T: Iterator<Item=&'a Step>> Iterator for CoalescedStepIter<'a, T> {
    type Item = CoalescedStep;

    fn next(&mut self) -> Option<Self::Item> {
        let mut coalesced = match self.iter.next() {
            None => { return None; }
            Some(x) => CoalescedStep {
                start: self.offset,
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
                        Op::GapFirst => { self.offset.seq1 += coalesced.len; }
                        Op::GapSecond => { self.offset.seq2 += coalesced.len; }
                        Op::Equivalent | Op::Match | Op::Mismatch => {
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


pub struct Alignment<S: scoring::Score> {
    pub score: S,
    pub steps: Vec<Step>,
    pub seq1: Range<usize>,
    pub seq2: Range<usize>,
}

impl<S: scoring::Score> Alignment<S> {
    pub fn len(&self) -> usize {
        self.steps.iter().map(|x| x.len as usize).sum()
    }

    pub fn rle(&self) -> String {
        utils::rle(&self.steps, self.len())
    }

    pub fn prettify(&self, seq1: &str, seq2: &str) -> String {
        let seq1 = &seq1[self.seq1.start..self.seq1.end];
        let seq2 = &seq2[self.seq2.start..self.seq2.end];
        let total: usize = self.len();
        utils::prettify(seq1, seq2, &self.steps, total)
    }

    pub fn intersects(&self, other: &Alignment<S>) -> bool {
        if max(self.seq1.start, other.seq1.start) >= min(self.seq1.end, other.seq1.end) {
            return false;
        }
        if max(self.seq2.start, other.seq2.start) >= min(self.seq2.end, other.seq2.end) {
            return false;
        }
        return utils::intersects(self.coalesced_steps(), other.coalesced_steps());
    }

    pub fn coalesced_steps(&self) -> impl Iterator<Item=CoalescedStep> + '_ {
        CoalescedStepIter {
            iter: self.steps.iter().peekable(),
            offset: Offset { seq1: self.seq1.start, seq2: self.seq2.start },
        }
    }
}
