use std::fmt::{Debug, Display, Formatter};

use derive_getters::{Dissolve, Getters};
use derive_more::{From, Into};
use eyre::Result;
use itertools::{chain, Itertools};

use biobit_core_rs::loc::{AsSegment, Segment};
use biobit_core_rs::num::PrimInt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Getters, Dissolve)]
pub struct InvSegment<Idx: PrimInt> {
    left: Segment<Idx>,
    right: Segment<Idx>,
}

impl<Idx: PrimInt> InvSegment<Idx> {
    pub fn new(left: Segment<Idx>, right: Segment<Idx>) -> Result<Self> {
        if left.len() != right.len() {
            return Err(eyre::eyre!(
                "Repeat segments' length must be equal: {left:?} vs {right:?}"
            ));
        }
        if left.intersects(&right) {
            return Err(eyre::eyre!(
                "Repeat segments must not overlap: {left:?} vs {right:?}"
            ));
        }

        Ok(Self { left, right })
    }

    pub fn inner_gap(&self) -> Idx {
        self.right().start() - self.left().end()
    }

    pub fn seqlen(&self) -> Idx {
        self.left().len().shl(1) // = len * 2
    }

    pub fn brange(&self) -> Segment<Idx> {
        Segment::new(self.left().start(), self.right().end()).unwrap()
    }

    pub fn len(&self) -> Idx {
        self.left.len()
    }

    pub fn shift(&mut self, shift: Idx) {
        self.left.shift(shift);
        self.right.shift(shift);
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

#[derive(Eq, PartialEq, Hash, Clone, Getters, Dissolve)]
pub struct InvRepeat<Idx: PrimInt> {
    segments: Vec<InvSegment<Idx>>,
}

impl<Idx: PrimInt> InvRepeat<Idx> {
    pub fn new(segments: Vec<InvSegment<Idx>>) -> Result<Self> {
        if segments.is_empty() {
            return Err(eyre::eyre!(
                "Inverted repeat must have at least one segment"
            ));
        }

        for (prev, nxt) in segments.iter().tuple_windows() {
            if prev.left.end() > nxt.left.start() || prev.right.start() < nxt.right.end() {
                return Err(eyre::eyre!(
                    "Segments must be ordered from outer to inner and must not overlap: {prev:?} vs {nxt:?}"
                ));
            }
        }
        Ok(Self { segments })
    }

    pub fn seqlen(&self) -> Idx {
        self.segments
            .iter()
            .fold(Idx::zero(), |acc, x| acc + x.seqlen())
    }

    pub fn inner_gap(&self) -> Idx {
        self.segments().last().unwrap().inner_gap()
    }

    pub fn left_brange(&self) -> Segment<Idx> {
        Segment::new(
            self.segments()[0].left().start(),
            self.segments().last().unwrap().left().end(),
        )
        .unwrap()
    }

    pub fn right_brange(&self) -> Segment<Idx> {
        Segment::new(
            self.segments().last().unwrap().right().start(),
            self.segments()[0].right().end(),
        )
        .unwrap()
    }

    pub fn brange(&self) -> Segment<Idx> {
        self.segments()[0].brange()
    }

    pub fn shift(&mut self, shift: Idx) {
        for x in &mut self.segments {
            x.shift(shift)
        }
    }

    pub fn seqranges(&self) -> impl Iterator<Item = &'_ Segment<Idx>> {
        chain(
            self.segments.iter().map(|x| x.left()),
            self.segments.iter().rev().map(|x| x.right()),
        )
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
