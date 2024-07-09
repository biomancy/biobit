use std::fmt::Display;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use derive_getters::Dissolve;
use impl_tools::autoimpl;
use num::{NumCast, Unsigned};

use crate::num::PrimInt;

/// Segment is a half-open genomic region [start, end).
/// It's not represented as a Rust-native Range for a couple of reasons:
/// - Prohibit 'empty' segments (start == end) or segments with negative length (start > end)
/// - Implement custom traits (e.g. Dissolve) and methods (e.g. contains, intersects, touches).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve)]
pub struct Segment<Idx: PrimInt> {
    start: Idx,
    end: Idx,
}

/// Trait for types that can be generally viewed as half-open genomic segments [start, end).
#[autoimpl(for < T: trait + ? Sized > & T, Box < T >, Rc < T >, Arc < T >)]
pub trait SegmentLike {
    type Idx: PrimInt;

    /// Start position of the segment-like object.
    fn start(&self) -> Self::Idx;

    /// End position of the segment-like object.
    fn end(&self) -> Self::Idx;

    /// Length of the segment-like object.
    fn len(&self) -> Self::Idx {
        self.end() - self.start()
    }

    /// Check if the segment-like object contains a given position.
    fn contains(&self, pos: Self::Idx) -> bool {
        self.start() <= pos && pos < self.end()
    }

    /// Check if the segment-like object intersects with another segment-like object.
    /// The condition is strict and doesn't allow touching segments.
    fn intersects(&self, other: &Self) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }

    /// Check if the segment-like object touches another segment-like object.
    /// The condition is strict and should not allow overlapping segments.
    fn touches(&self, other: &Self) -> bool {
        self.start() == other.end() || self.end() == other.start()
    }

    /// Turn the segment-like object into a basic half-open genomic segment.
    fn as_segment(&self) -> Segment<Self::Idx> {
        Segment {
            start: self.start(),
            end: self.end(),
        }
    }
}

impl<T: PrimInt> SegmentLike for Segment<T> {
    type Idx = T;

    #[inline(always)]
    fn start(&self) -> Self::Idx {
        self.start
    }
    #[inline(always)]
    fn end(&self) -> Self::Idx {
        self.end
    }
}

impl<Idx: PrimInt> Segment<Idx> {
    pub fn new(start: Idx, end: Idx) -> Result<Self, ()> {
        if start < end {
            Ok(Self { start, end })
        } else {
            Err(())
        }
    }

    pub fn extend<T: Unsigned + NumCast>(&mut self, left: T, right: T) -> Option<&mut Self> {
        match (num::cast(left), num::cast(right)) {
            (Some(left), Some(right)) => {
                self.start = self.start - left;
                self.end = self.end + right;
                Some(self)
            }
            _ => None,
        }
    }

    pub unsafe fn extend_unchecked<T: Unsigned + NumCast>(
        &mut self,
        left: T,
        right: T,
    ) -> &mut Self {
        self.start = self.start - num::cast(left).unwrap_unchecked();
        self.end = self.end + num::cast(right).unwrap_unchecked();
        self
    }

    pub fn extended<T: Unsigned + NumCast>(&self, left: T, right: T) -> Option<Self> {
        match (num::cast(left), num::cast(right)) {
            (Some(left), Some(right)) => Some(Self {
                start: self.start - left,
                end: self.end + right,
            }),
            _ => None,
        }
    }

    pub unsafe fn extended_unchecked<T: Unsigned + NumCast>(&self, left: T, right: T) -> Self {
        Self {
            start: self.start - num::cast(left).unwrap_unchecked(),
            end: self.end + num::cast(right).unwrap_unchecked(),
        }
    }

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub fn union(&self, other: &Self) -> Option<Self> {
        match self.intersects(other) || self.touches(other) {
            true => Some(Self {
                start: self.start.min(other.start),
                end: self.end.max(other.end),
            }),
            false => None,
        }
    }
}

impl<Idx: PrimInt> Default for Segment<Idx> {
    fn default() -> Self {
        Self {
            start: Idx::zero(),
            end: Idx::one(),
        }
    }
}

impl<Idx: PrimInt + Display> Display for Segment<Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

impl<Idx: PrimInt> TryFrom<(Idx, Idx)> for Segment<Idx> {
    type Error = ();

    fn try_from(value: (Idx, Idx)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1)
    }
}

impl<Idx: PrimInt> TryFrom<Range<Idx>> for Segment<Idx> {
    type Error = ();

    fn try_from(value: Range<Idx>) -> Result<Self, Self::Error> {
        Self::new(value.start, value.end)
    }
}

impl<Idx: PrimInt> From<Segment<Idx>> for Range<Idx> {
    fn from(segment: Segment<Idx>) -> Self {
        segment.start..segment.end
    }
}

impl<Idx: PrimInt> From<&Segment<Idx>> for Range<Idx> {
    fn from(segment: &Segment<Idx>) -> Self {
        segment.start..segment.end
    }
}

impl<Idx: PrimInt> PartialEq<Range<Idx>> for Segment<Idx> {
    fn eq(&self, other: &Range<Idx>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<Idx: PrimInt> PartialEq<Segment<Idx>> for Range<Idx> {
    fn eq(&self, other: &Segment<Idx>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct() {
        assert_eq!(Segment::new(0, 10), Ok(Segment { start: 0, end: 10 }));
        assert_eq!(Segment::new(1, 0), Err(()));
        assert_eq!(Segment::new(0, 0), Err(()));
    }

    #[test]
    fn test_len() {
        assert_eq!(Segment::new(0, 10).unwrap().len(), 10);
        assert_eq!(Segment::new(0, 1).unwrap().len(), 1);
    }

    #[test]
    fn test_contains() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(segment.contains(0), false);
        assert_eq!(segment.contains(1), true);
        assert_eq!(segment.contains(5), true);
        assert_eq!(segment.contains(9), true);
        assert_eq!(segment.contains(10), false);
        assert_eq!(segment.contains(11), false);
    }

    #[test]
    fn test_intersects() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(segment.intersects(&Segment::new(0, 1).unwrap()), false);
        assert_eq!(segment.intersects(&Segment::new(0, 2).unwrap()), true);
        assert_eq!(segment.intersects(&Segment::new(5, 9).unwrap()), true);
        assert_eq!(segment.intersects(&Segment::new(9, 10).unwrap()), true);
        assert_eq!(segment.intersects(&Segment::new(10, 11).unwrap()), false);
    }

    #[test]
    fn test_touches() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(segment.touches(&Segment::new(0, 1).unwrap()), true);
        assert_eq!(segment.touches(&Segment::new(0, 2).unwrap()), false);
        assert_eq!(segment.touches(&Segment::new(5, 9).unwrap()), false);
        assert_eq!(segment.touches(&Segment::new(9, 10).unwrap()), false);
        assert_eq!(segment.touches(&Segment::new(10, 11).unwrap()), true);
    }

    #[test]
    fn test_extend() {
        let mut segment = Segment::new(1, 10).unwrap();
        assert_eq!(
            segment.extend(1u8, 2u8),
            Some(&mut Segment { start: 0, end: 12 })
        );
        assert_eq!(
            segment.extend(1usize, 0usize),
            Some(&mut Segment { start: -1, end: 12 })
        );
    }

    #[test]
    fn test_extend_unchecked() {
        let mut segment = Segment::new(1, 10).unwrap();

        assert_eq!(
            unsafe { segment.extend_unchecked(1u8, 2u8) },
            &mut Segment { start: 0, end: 12 }
        );
        assert_eq!(
            unsafe { segment.extend_unchecked(1usize, 0usize) },
            &mut Segment { start: -1, end: 12 }
        );
    }

    #[test]
    fn test_extended() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(
            segment.extended(1u8, 2u8),
            Some(Segment { start: 0, end: 12 })
        );
        assert_eq!(
            segment.extended(1usize, 0usize),
            Some(Segment { start: 0, end: 10 })
        );
    }

    #[test]
    fn test_extended_unchecked() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(
            unsafe { segment.extended_unchecked(1u8, 2u8) },
            Segment { start: 0, end: 12 }
        );
        assert_eq!(
            unsafe { segment.extended_unchecked(1usize, 0usize) },
            Segment { start: 0, end: 10 }
        );
    }

    #[test]
    fn test_intersection() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(segment.intersection(&Segment::new(0, 1).unwrap()), None);
        assert_eq!(
            segment.intersection(&Segment::new(0, 2).unwrap()),
            Some(Segment { start: 1, end: 2 })
        );
        assert_eq!(
            segment.intersection(&Segment::new(5, 9).unwrap()),
            Some(Segment { start: 5, end: 9 })
        );
        assert_eq!(
            segment.intersection(&Segment::new(9, 11).unwrap()),
            Some(Segment { start: 9, end: 10 })
        );
        assert_eq!(segment.intersection(&Segment::new(10, 11).unwrap()), None);
    }

    #[test]
    fn test_union() {
        let segment = Segment::new(1, 10).unwrap();
        assert_eq!(
            segment.union(&Segment::new(0, 1).unwrap()),
            Some(Segment { start: 0, end: 10 })
        );
        assert_eq!(
            segment.union(&Segment::new(0, 2).unwrap()),
            Some(Segment { start: 0, end: 10 })
        );
        assert_eq!(
            segment.union(&Segment::new(5, 9).unwrap()),
            Some(Segment { start: 1, end: 10 })
        );
        assert_eq!(
            segment.union(&Segment::new(9, 11).unwrap()),
            Some(Segment { start: 1, end: 11 })
        );
        assert_eq!(segment.union(&Segment::new(-1, 0).unwrap()), None);
        assert_eq!(segment.union(&Segment::new(11, 12).unwrap()), None);
    }
}
