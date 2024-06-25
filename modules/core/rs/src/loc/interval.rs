use std::ops::Range;

use derive_getters::Dissolve;
use num::{FromPrimitive, NumCast, ToPrimitive, Unsigned};
use num::traits::AsPrimitive;

use crate::num::{PrimInt, PrimUInt};

/// Interval is a half-open genomic region [start, end).
/// It's not represented as a Rust-native Range for a few reasons:
/// - Prohibit 'empty' intervals (start == end) or intervals with negative length (start > end)
/// - Implement custom traits (e.g. Dissolve) and methods (e.g. contains, intersects, touches).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve)]
pub struct Interval<Idx: PrimInt> {
    pub start: Idx,
    pub end: Idx,
}

/// Trait for types that can be generally viewed as half-open genomic intervals [start, end).
pub trait LikeInterval: Into<Interval<Self::Idx>> {
    type Idx: PrimInt;

    /// Start position of the interval-like object.
    fn start(&self) -> Self::Idx;

    /// End position of the interval-like object.
    fn end(&self) -> Self::Idx;

    /// Length of the interval-like object.
    fn len(&self) -> Self::Idx { self.end() - self.start() }

    /// Check if the interval-like object contains a given position.
    fn contains(&self, pos: Self::Idx) -> bool { self.start() <= pos && pos < self.end() }

    /// Check if the interval-like object intersects with another interval-like object.
    /// The condition is strict and doesn't allow touching intervals.
    fn intersects(&self, other: &Self) -> bool { self.start() < other.end() && other.start() < self.end() }

    /// Check if the interval-like object touches another interval-like object.
    /// The condition is strict and should not allow overlapping intervals.
    fn touches(&self, other: &Self) -> bool { self.start() == other.end() || self.end() == other.start() }

    /// Turn the interval-like object into a basic half-open genomic interval.
    fn as_interval(&self) -> Interval<Self::Idx> { Interval { start: self.start(), end: self.end() } }
}

impl<T: PrimInt> LikeInterval for Interval<T> {
    type Idx = T;

    #[inline(always)]
    fn start(&self) -> Self::Idx { self.start }
    #[inline(always)]
    fn end(&self) -> Self::Idx { self.end }
}


impl<Idx: PrimInt> Interval<Idx> {
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
            _ => None
        }
    }

    pub unsafe fn extend_unchecked<T: Unsigned + NumCast>(&mut self, left: T, right: T) -> &mut Self {
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
            _ => None
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
            false => None
        }
    }

    // pub fn split(&self, pos: Idx) -> Option<(Self, Self)> {
    //     if self.start < pos && pos < self.end {
    //         Some((Self { start: self.start, end: pos }, Self { start: pos, end: self.end }))
    //     } else {
    //         None
    //     }
    // }
}

impl<Idx: PrimInt> Default for Interval<Idx> {
    fn default() -> Self {
        Self { start: Idx::zero(), end: Idx::one() }
    }
}

impl<Idx: PrimInt> TryFrom<(Idx, Idx)> for Interval<Idx> {
    type Error = ();

    fn try_from(value: (Idx, Idx)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1)
    }
}

impl<Idx: PrimInt> TryFrom<Range<Idx>> for Interval<Idx> {
    type Error = ();

    fn try_from(value: Range<Idx>) -> Result<Self, Self::Error> {
        Self::new(value.start, value.end)
    }
}


mod tests {
    use super::*;

    #[test]
    fn test_construct() {
        assert_eq!(Interval::new(0, 10), Ok(Interval { start: 0, end: 10 }));
        assert_eq!(Interval::new(1, 0), Err(()));
        assert_eq!(Interval::new(0, 0), Err(()));
    }

    #[test]
    fn test_len() {
        assert_eq!(Interval::new(0, 10).unwrap().len(), 10);
        assert_eq!(Interval::new(0, 1).unwrap().len(), 1);
    }

    #[test]
    fn test_contains() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(interval.contains(0), false);
        assert_eq!(interval.contains(1), true);
        assert_eq!(interval.contains(5), true);
        assert_eq!(interval.contains(9), true);
        assert_eq!(interval.contains(10), false);
        assert_eq!(interval.contains(11), false);
    }

    #[test]
    fn test_intersects() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(interval.intersects(&Interval::new(0, 1).unwrap()), false);
        assert_eq!(interval.intersects(&Interval::new(0, 2).unwrap()), true);
        assert_eq!(interval.intersects(&Interval::new(5, 9).unwrap()), true);
        assert_eq!(interval.intersects(&Interval::new(9, 10).unwrap()), true);
        assert_eq!(interval.intersects(&Interval::new(10, 11).unwrap()), false);
    }

    #[test]
    fn test_touches() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(interval.touches(&Interval::new(0, 1).unwrap()), true);
        assert_eq!(interval.touches(&Interval::new(0, 2).unwrap()), false);
        assert_eq!(interval.touches(&Interval::new(5, 9).unwrap()), false);
        assert_eq!(interval.touches(&Interval::new(9, 10).unwrap()), false);
        assert_eq!(interval.touches(&Interval::new(10, 11).unwrap()), true);
    }

    #[test]
    fn test_extend() {
        let mut interval = Interval::new(1, 10).unwrap();
        assert_eq!(
            interval.extend(1u8, 2u8),
            Some(&mut Interval { start: 0, end: 12 })
        );
        assert_eq!(
            interval.extend(1usize, 0usize),
            Some(&mut Interval { start: -1, end: 12 })
        );
    }

    #[test]
    fn test_extend_unchecked() {
        let mut interval = Interval::new(1, 10).unwrap();

        assert_eq!(
            unsafe { interval.extend_unchecked(1u8, 2u8) },
            &mut Interval { start: 0, end: 12 }
        );
        assert_eq!(
            unsafe { interval.extend_unchecked(1usize, 0usize) },
            &mut Interval { start: -1, end: 12 }
        );
    }

    #[test]
    fn test_extended() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(
            interval.extended(1u8, 2u8),
            Some(Interval { start: 0, end: 12 })
        );
        assert_eq!(
            interval.extended(1usize, 0usize),
            Some(Interval { start: 0, end: 10 })
        );
    }

    #[test]
    fn test_extended_unchecked() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(
            unsafe { interval.extended_unchecked(1u8, 2u8) },
            Interval { start: 0, end: 12 }
        );
        assert_eq!(
            unsafe { interval.extended_unchecked(1usize, 0usize) },
            Interval { start: 0, end: 10 }
        );
    }

    #[test]
    fn test_intersection() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(
            interval.intersection(&Interval::new(0, 1).unwrap()),
            None
        );
        assert_eq!(
            interval.intersection(&Interval::new(0, 2).unwrap()),
            Some(Interval { start: 1, end: 2 })
        );
        assert_eq!(
            interval.intersection(&Interval::new(5, 9).unwrap()),
            Some(Interval { start: 5, end: 9 })
        );
        assert_eq!(
            interval.intersection(&Interval::new(9, 11).unwrap()),
            Some(Interval { start: 9, end: 10 })
        );
        assert_eq!(
            interval.intersection(&Interval::new(10, 11).unwrap()),
            None
        );
    }

    #[test]
    fn test_union() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(
            interval.union(&Interval::new(0, 1).unwrap()),
            Some(Interval { start: 0, end: 10 })
        );
        assert_eq!(
            interval.union(&Interval::new(0, 2).unwrap()),
            Some(Interval { start: 0, end: 10 })
        );
        assert_eq!(
            interval.union(&Interval::new(5, 9).unwrap()),
            Some(Interval { start: 1, end: 10 })
        );
        assert_eq!(
            interval.union(&Interval::new(9, 11).unwrap()),
            Some(Interval { start: 1, end: 11 })
        );
        assert_eq!(
            interval.union(&Interval::new(-1, 0).unwrap()),
            None
        );
        assert_eq!(
            interval.union(&Interval::new(11, 12).unwrap()),
            None
        );
    }
}