use std::fmt::{Debug, Formatter};
use std::ops::Range;

use derive_getters::{Dissolve, Getters};
use itertools::{chain, Itertools};

pub use super::Coordinate;

#[derive(Eq, PartialEq, Hash, Clone, Getters, Dissolve)]
pub struct Segment<Idx: Coordinate> {
    left: Range<Idx>,
    right: Range<Idx>,
}

impl<Idx: Coordinate> Segment<Idx> {
    pub fn new(left: Range<Idx>, right: Range<Idx>) -> Self
        where Idx: Debug
    {
        assert!(left.start < left.end, "Sequence range start must be < end: {left:?}");
        assert!(right.start < right.end, "Sequence range start must be < end: {right:?}");

        assert_eq!(
            left.end - left.start, right.end - right.start,
            "Repeat segments' length must be equal: {left:?} vs {right:?}"
        );
        assert!(
            left.start < left.end && left.end <= right.start && right.start < right.end,
            "Repeat segments must not overlap: {left:?} vs {right:?}"
        );
        Self { left, right }
    }

    fn inner_gap(&self) -> Idx { self.right().start - self.left().end }

    fn seqlen(&self) -> Idx { (self.left().end - self.left().start).shl(1) }

    fn brange(&self) -> Range<Idx> { self.left().start..self.right().end }

    fn shift(&mut self, shift: &Idx) {
        self.left.start += *shift;
        self.left.end += *shift;

        self.right.start += *shift;
        self.right.end += *shift;
    }
}


impl<Idx: Coordinate + Debug> Debug for Segment<Idx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "inv::Segment [{:?}-{:?}) <=> [{:?}-{:?})",
            self.left.start, self.left.end, self.right.start, self.right.end
        )
    }
}

impl<Idx: Coordinate> From<(Range<Idx>, Range<Idx>)> for Segment<Idx> {
    fn from(value: (Range<Idx>, Range<Idx>)) -> Self {
        Self { left: value.0, right: value.1 }
    }
}


#[derive(Eq, PartialEq, Hash, Clone, Getters, Dissolve)]
pub struct Repeat<Idx: Coordinate> {
    segments: Vec<Segment<Idx>>,
}

impl<Idx: Coordinate> Repeat<Idx> {
    pub fn new(segments: Vec<Segment<Idx>>) -> Self
        where Idx: Debug
    {
        assert!(!segments.is_empty(), "Inverted repeat must have at least one segment");
        // segments.sort_by_key(|x| x.left.start);

        for (prev, nxt) in segments.iter().tuple_windows() {
            assert!(
                (prev.left.end <= nxt.left.start) && (prev.right.start >= nxt.right.end),
                "Segments must be ordered from outer to inner and must not overlap: {prev:?} vs {nxt:?}"
            );
        }

        Self { segments }
    }

    pub fn seqlen(&self) -> Idx {
        let mut seqlen = Idx::zero();
        for s in self.segments().iter().map(|x| x.seqlen()) {
            seqlen += s;
        }
        return seqlen;
    }

    pub fn inner_gap(&self) -> Idx { self.segments().last().unwrap().inner_gap() }

    pub fn left_brange(&self) -> Range<Idx> {
        Range {
            start: self.segments()[0].left().start,
            end: self.segments().last().unwrap().left().end,
        }
    }

    pub fn right_brange(&self) -> Range<Idx> {
        Range {
            start: self.segments().last().unwrap().right().start,
            end: self.segments()[0].right().end,
        }
    }

    pub fn brange(&self) -> Range<Idx> { self.segments()[0].brange() }

    pub fn shift(&mut self, shift: &Idx) {
        for x in &mut self.segments { x.shift(shift) }
    }

    pub fn seqranges(&self) -> impl Iterator<Item=&'_ Range<Idx>>
    {
        chain(
            self.segments().iter().map(|x| x.left()),
            self.segments().iter().rev().map(|x| x.right()),
        )
    }
}

impl<Idx: Coordinate + Debug> Debug for Repeat<Idx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "inv::Repeat [{:?}-{:?}) <=> [{:?}-{:?})",
            self.left_brange().start, self.left_brange().end,
            self.right_brange().start, self.right_brange().end
        )
    }
}
