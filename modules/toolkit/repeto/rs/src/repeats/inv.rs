use std::fmt::{Debug, Display, Formatter};

use derive_getters::{Dissolve, Getters};
use derive_more::From;
use eyre::{Result, ensure};
use itertools::{Itertools, chain};

use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimInt;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Dissolve, Getters)]
pub struct InvSegment<Idx: PrimInt> {
    left: Interval<Idx>,
    right: Interval<Idx>,
}

impl<Idx: PrimInt> InvSegment<Idx> {
    pub fn new(left: Interval<Idx>, right: Interval<Idx>) -> Result<Self> {
        ensure!(
            left.len() == right.len() && !left.intersects(&right) && left.end() <= right.start(),
            "Inverted segment must have equal length and not overlap: {left:?} vs {right:?}"
        );
        Ok(Self { left, right })
    }

    pub fn inner_gap(&self) -> Idx {
        self.right().start() - self.left().end()
    }

    pub fn brange(&self) -> Interval<Idx> {
        Interval::new(self.left().start(), self.right().end()).unwrap()
    }
    pub fn len(&self) -> Idx {
        self.left.len()
    }

    pub fn seqlen(&self) -> Idx {
        self.len().shl(1) // = len * 2
    }

    pub fn shift(&mut self, shift: Idx) {
        self.left.shift(shift);
        self.right.shift(shift);
    }

    pub fn cast<T: PrimInt>(&self) -> Option<InvSegment<T>> {
        match (self.left.cast::<T>(), self.right.cast::<T>()) {
            (Some(left), Some(right)) => Some(InvSegment { left, right }),
            _ => None,
        }
    }
}

impl<Idx: PrimInt> Debug for InvSegment<Idx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvSegment {:?} <=> {:?}", self.left, self.right)
    }
}

impl<Idx: PrimInt + Display> Display for InvSegment<Idx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvSegment {} <=> {}", self.left, self.right)
    }
}

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Eq, PartialEq, Hash, Clone, Getters, Dissolve)]
pub struct InvRepeat<Idx: PrimInt> {
    segments: Vec<InvSegment<Idx>>,
}

impl<Idx: PrimInt> InvRepeat<Idx> {
    pub fn new(intervals: Vec<InvSegment<Idx>>) -> Result<Self> {
        if intervals.is_empty() {
            return Err(eyre::eyre!(
                "Inverted repeat must have at least one segment"
            ));
        }

        for (prev, nxt) in intervals.iter().tuple_windows() {
            if prev.left.end() > nxt.left.start() || prev.right.start() < nxt.right.end() {
                return Err(eyre::eyre!(
                    "Intervals must be ordered from outer to inner and must not overlap: {prev:?} vs {nxt:?}"
                ));
            }
        }
        Ok(Self {
            segments: intervals,
        })
    }

    pub fn len(&self) -> Idx {
        self.segments
            .iter()
            .map(|x| x.len())
            .fold(Idx::zero(), |a, b| a + b)
    }

    pub fn seqlen(&self) -> Idx {
        self.len().shl(1) // = len * 2
    }

    pub fn inner_gap(&self) -> Idx {
        self.segments().last().unwrap().inner_gap()
    }

    pub fn left_brange(&self) -> Interval<Idx> {
        Interval::new(
            self.segments()[0].left().start(),
            self.segments().last().unwrap().left().end(),
        )
        .unwrap()
    }

    pub fn right_brange(&self) -> Interval<Idx> {
        Interval::new(
            self.segments().last().unwrap().right().start(),
            self.segments()[0].right().end(),
        )
        .unwrap()
    }

    pub fn brange(&self) -> Interval<Idx> {
        self.segments()[0].brange()
    }

    pub fn shift(&mut self, shift: Idx) {
        for x in &mut self.segments {
            x.shift(shift)
        }
    }

    pub fn seqranges(&self) -> impl Iterator<Item = &'_ Interval<Idx>> {
        chain(
            self.segments.iter().map(|x| x.left()),
            self.segments.iter().rev().map(|x| x.right()),
        )
    }

    pub fn cast<T: PrimInt>(&self) -> Option<InvRepeat<T>> {
        self.segments
            .iter()
            .map(|x| x.cast::<T>())
            .collect::<Option<Vec<_>>>()
            .map(|segments| InvRepeat { segments })
    }
}

impl<Idx: PrimInt + Debug> Debug for InvRepeat<Idx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "inv::Repeat {:?} <=> {:?}",
            self.left_brange(),
            self.right_brange(),
        )
    }
}
