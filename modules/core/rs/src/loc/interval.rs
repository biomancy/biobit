use std::fmt::{Debug, Display};
use std::ops::{Range, Shl, Shr};
use std::rc::Rc;
use std::sync::Arc;

use crate::num::PrimInt;
use bitcode::{Decode, Encode};
use derive_getters::Dissolve;
use eyre::{eyre, Report, Result};
use impl_tools::autoimpl;
use num::{NumCast, Unsigned};

/// Interval is a half-open genomic region [start, end).
/// It's not represented as a Rust-native Range for a couple of reasons:
/// - Prohibit 'empty' intervals (start == end) or intervals with negative length (start > end)
/// - Implement custom traits (e.g. Dissolve) and methods (e.g. contains, intersects, touches).
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve)]
pub struct Interval<Idx: PrimInt> {
    start: Idx,
    end: Idx,
}

/// Trait for types that can be generally viewed as half-open genomic intervals [start, end).
#[autoimpl(for <T: trait + ?Sized> &T, Box<T>, Rc<T>, Arc<T>)]
#[allow(clippy::len_without_is_empty)]
pub trait IntervalOp {
    type Idx: PrimInt;

    /// Start position of the interval-like object.
    fn start(&self) -> Self::Idx;

    /// End position of the interval-like object.
    fn end(&self) -> Self::Idx;

    /// Length of the interval-like object.
    fn len(&self) -> Self::Idx {
        self.end() - self.start()
    }

    /// Check if the interval-like object contains a given position.
    fn contains(&self, pos: Self::Idx) -> bool {
        self.start() <= pos && pos < self.end()
    }

    /// Check if the interval-like object intersects with another interval-like object.
    /// The condition is strict and doesn't allow touching intervals.
    fn intersects(&self, other: &Self) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }

    /// Check if the interval-like object touches another interval-like object.
    /// The condition is strict and should not allow overlapping intervals.
    fn touches(&self, other: &Self) -> bool {
        self.start() == other.end() || self.end() == other.start()
    }

    /// Turn the interval-like object into a basic half-open genomic interval.
    fn as_interval(&self) -> Interval<Self::Idx> {
        Interval {
            start: self.start(),
            end: self.end(),
        }
    }
}

impl<T: PrimInt> IntervalOp for Interval<T> {
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

impl<Idx: PrimInt> Interval<Idx> {
    pub fn new(start: Idx, end: Idx) -> Result<Self> {
        if start < end {
            Ok(Self { start, end })
        } else {
            Err(eyre!("Invalid interval: start >= end"))
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
    /// # Safety
    ///
    /// This function is unsafe because it doesn't check if the resulting interval is valid.
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

    /// # Safety
    ///
    /// This function is unsafe because it doesn't check if the resulting interval is valid.
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

    pub fn intersection_length(&self, other: &Self) -> Idx {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start < end {
            end - start
        } else {
            Idx::zero()
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

    pub fn merge(intervals: &mut [Self]) -> Vec<Self> {
        // TODO: make it much more efficient and API-friendly. Why do I have union and merge? Can I merge in-place? Can I merge in a single pass? Do I need a separate namespace for this?
        if intervals.is_empty() {
            return Vec::new();
        }
        intervals.sort_by_key(|x| x.start());

        let mut merged = Vec::new();
        let (mut start, mut end) = (intervals[0].start(), intervals[0].end());
        for current in intervals {
            if current.start() > end {
                merged.push(Interval::new(start, end).unwrap());
                end = current.end();
                start = current.start();
            } else if current.end() > end {
                end = current.end();
            }
        }
        merged.push(Interval::new(start, end).unwrap());

        merged
    }

    pub fn merge_within(intervals: &mut [Self], distance: Idx) -> Vec<Self> {
        if intervals.is_empty() {
            return Vec::new();
        }
        intervals.sort_by_key(|x| x.start());

        let mut iter = intervals.iter();
        let mut merged = Vec::new();
        let (mut start, mut end) = iter.next().unwrap().dissolve();
        for current in iter {
            if current.start() > end + distance {
                merged.push(Interval::new(start, end).unwrap());
                end = current.end();
                start = current.start();
            } else if current.end() > end {
                end = current.end();
            }
        }
        merged.push(Interval::new(start, end).unwrap());

        merged
    }

    pub fn shift(&mut self, shift: Idx) -> &mut Self {
        self.start = self.start + shift;
        self.end = self.end + shift;
        self
    }

    pub fn clamped(self, inside: &Self) -> Option<Self> {
        self.intersection(inside)
    }

    pub fn cast<T: PrimInt>(&self) -> Option<Interval<T>> {
        match (T::from(self.start), T::from(self.end)) {
            (Some(start), Some(end)) => Some(Interval { start, end }),
            _ => None,
        }
    }
}

impl<Idx: PrimInt> Default for Interval<Idx> {
    fn default() -> Self {
        Self {
            start: Idx::zero(),
            end: Idx::one(),
        }
    }
}

impl<Idx: PrimInt + Display> Display for Interval<Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

impl<Idx: PrimInt> TryFrom<(Idx, Idx)> for Interval<Idx> {
    type Error = Report;

    fn try_from(value: (Idx, Idx)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1)
    }
}

impl<Idx: PrimInt> From<Interval<Idx>> for (Idx, Idx) {
    fn from(interval: Interval<Idx>) -> Self {
        (interval.start, interval.end)
    }
}

impl<Idx: PrimInt> TryFrom<Range<Idx>> for Interval<Idx> {
    type Error = Report;

    fn try_from(value: Range<Idx>) -> Result<Self, Self::Error> {
        Self::new(value.start, value.end)
    }
}

impl<Idx: PrimInt> From<Interval<Idx>> for Range<Idx> {
    fn from(interval: Interval<Idx>) -> Self {
        interval.start..interval.end
    }
}

impl<Idx: PrimInt> From<&Interval<Idx>> for Range<Idx> {
    fn from(interval: &Interval<Idx>) -> Self {
        interval.start..interval.end
    }
}

impl<Idx: PrimInt> PartialEq<(Idx, Idx)> for Interval<Idx> {
    fn eq(&self, other: &(Idx, Idx)) -> bool {
        self.start == other.0 && self.end == other.1
    }
}

impl<Idx: PrimInt> PartialEq<Range<Idx>> for Interval<Idx> {
    fn eq(&self, other: &Range<Idx>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<Idx: PrimInt> PartialEq<Interval<Idx>> for Range<Idx> {
    fn eq(&self, other: &Interval<Idx>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<Idx: PrimInt> Shl<Idx> for Interval<Idx> {
    type Output = Self;

    fn shl(mut self, shift: Idx) -> Self::Output {
        self.start = self.start - shift;
        self.end = self.end - shift;
        self
    }
}

impl<Idx: PrimInt> Shl<Idx> for &Interval<Idx> {
    type Output = Interval<Idx>;

    fn shl(self, shift: Idx) -> Self::Output {
        Interval {
            start: self.start - shift,
            end: self.end - shift,
        }
    }
}

impl<Idx: PrimInt> Shl<Idx> for &mut Interval<Idx> {
    type Output = Self;

    fn shl(self, shift: Idx) -> Self::Output {
        self.start = self.start - shift;
        self.end = self.end - shift;
        self
    }
}

impl<Idx: PrimInt> Shr<Idx> for Interval<Idx> {
    type Output = Self;

    fn shr(mut self, shift: Idx) -> Self::Output {
        self.start = self.start + shift;
        self.end = self.end + shift;
        self
    }
}

impl<Idx: PrimInt> Shr<Idx> for &Interval<Idx> {
    type Output = Interval<Idx>;

    fn shr(self, shift: Idx) -> Self::Output {
        Interval {
            start: self.start + shift,
            end: self.end + shift,
        }
    }
}

impl<Idx: PrimInt> Shr<Idx> for &mut Interval<Idx> {
    type Output = Self;

    fn shr(self, shift: Idx) -> Self::Output {
        self.start = self.start + shift;
        self.end = self.end + shift;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct() {
        assert_eq!(
            Interval::new(0, 10).unwrap(),
            Interval { start: 0, end: 10 }
        );
        assert!(Interval::new(1, 0).is_err());
        assert!(Interval::new(0, 0).is_err());
    }

    #[test]
    fn test_len() {
        assert_eq!(Interval::new(0, 10).unwrap().len(), 10);
        assert_eq!(Interval::new(0, 1).unwrap().len(), 1);
    }

    #[test]
    fn test_shift() {
        let interval = Interval::new(1, 10).unwrap();
        assert_eq!(interval >> 10, (11, 20));
        assert_eq!(interval << 1, (0, 9));
        assert_eq!((interval >> 10) << 10, interval);
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
        assert_eq!(interval.intersection(&Interval::new(0, 1).unwrap()), None);
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
        assert_eq!(interval.intersection(&Interval::new(10, 11).unwrap()), None);
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
        assert_eq!(interval.union(&Interval::new(-1, 0).unwrap()), None);
        assert_eq!(interval.union(&Interval::new(11, 12).unwrap()), None);
    }

    #[test]
    fn test_merge() {
        let mut intervals = vec![
            Interval::new(1, 10).unwrap(),
            Interval::new(5, 15).unwrap(),
            Interval::new(20, 30).unwrap(),
            Interval::new(25, 35).unwrap(),
        ];
        let merged = Interval::merge(&mut intervals);
        assert_eq!(
            merged,
            vec![
                Interval::new(1, 15).unwrap(),
                Interval::new(20, 35).unwrap(),
            ]
        );

        let merged = Interval::<isize>::merge(&mut vec![]);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_merge_within() {
        let mut intervals = vec![
            Interval::new(1, 10).unwrap(),
            Interval::new(5, 15).unwrap(),
            Interval::new(20, 30).unwrap(),
            Interval::new(25, 35).unwrap(),
            Interval::new(100, 200).unwrap(),
        ];
        assert_eq!(
            Interval::merge_within(&mut intervals, 0),
            Interval::merge(&mut intervals)
        );

        let merged = Interval::merge_within(&mut intervals, 5);
        assert_eq!(
            merged,
            vec![
                Interval::new(1, 35).unwrap(),
                Interval::new(100, 200).unwrap()
            ]
        );

        let merged = Interval::merge_within(&mut intervals, 100);
        assert_eq!(merged, vec![Interval::new(1, 200).unwrap()]);

        let merged = Interval::<isize>::merge_within(&mut vec![], 5);
        assert!(merged.is_empty());
    }
}
