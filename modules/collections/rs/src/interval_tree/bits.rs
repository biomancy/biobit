//! Implementation of an interval tree using the BITS algorithm.
//! Reference: https://doi.org/10.1093/bioinformatics/bts652

use super::results::{BatchHits, Hits};
use super::tree::{Builder, ITree};
use biobit_core_rs::{
    loc::{Interval, IntervalOp},
    num::PrimInt,
};
use derive_getters::Dissolve;
use derive_more::From;
use itertools::Itertools;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

/// A builder for constructing [`Bits`] interval trees.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, From, Dissolve)]
pub struct BitsBuilder<Idx: PrimInt, Data> {
    records: Vec<(Interval<Idx>, Data)>,
}

impl<Idx: PrimInt, Data> Default for BitsBuilder<Idx, Data> {
    fn default() -> Self {
        Self {
            records: Vec::new(),
        }
    }
}

impl<Idx: PrimInt, Data> Builder for BitsBuilder<Idx, Data> {
    type Target = Bits<Idx, Data>;

    fn add(
        mut self,
        interval: Interval<<Self::Target as ITree>::Idx>,
        data: <Self::Target as ITree>::Data,
    ) -> Self {
        self.records.push((interval, data));
        self
    }

    fn extend(
        mut self,
        records: impl IntoIterator<
            Item = (
                Interval<<Self::Target as ITree>::Idx>,
                <Self::Target as ITree>::Data,
            ),
        >,
    ) -> Self {
        self.records.extend(records.into_iter());
        self
    }

    fn build(self) -> Self::Target {
        Bits::new(self.records.into_iter())
    }
}

/// A partially immutable interval tree implementation using the BITS algorithm. The tree layout
/// is constant after construction, but the data elements can be modified.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Default, PartialEq, Eq, From, Dissolve)]
pub struct Bits<Idx: PrimInt, Data> {
    // Associated data elements, corresponding to intervals at the same index.
    data: Vec<Data>,
    // Interval start coordinates, sorted.
    starts: Vec<Idx>,
    // Interval end coordinates, corresponding to `starts`.
    ends: Vec<Idx>,
    // The maximum length of any interval in the tree. Used for query optimization.
    max_len: Idx,
}

impl<Idx: PrimInt, Data> Bits<Idx, Data> {
    /// Creates a new `Bits` interval tree from an iterator of `(Interval, Data)` pairs.
    /// All intervals are sorted internally by their start coordinates. Use `new_unchecked` for
    /// data that is already sorted.
    pub fn new(iter: impl IntoIterator<Item = (Interval<Idx>, Data)>) -> Self {
        let iter = iter.into_iter();

        let explen = iter.size_hint().0;
        let mut starts = Vec::with_capacity(explen);
        let mut ends = Vec::with_capacity(explen);
        let mut data = Vec::with_capacity(explen);
        let mut max_len = Idx::zero();

        for (interval, idata) in iter.sorted_by_key(|(it, _)| it.start()) {
            starts.push(interval.start());
            ends.push(interval.end());
            data.push(idata);
            max_len = max_len.max(interval.len());
        }

        Self {
            data,
            starts,
            ends,
            max_len,
        }
    }

    /// Creates a new `Bits` interval tree from an iterator of `(Interval, Data)` pairs.
    /// The intervals must be sorted by their start coordinates. Use `new` for unsorted data.
    pub unsafe fn new_unchecked(iter: impl IntoIterator<Item = (Interval<Idx>, Data)>) -> Self {
        let iter = iter.into_iter();

        let explen = iter.size_hint().0;
        let mut starts = Vec::with_capacity(explen);
        let mut ends = Vec::with_capacity(explen);
        let mut data = Vec::with_capacity(explen);
        let mut max_len = Idx::zero();

        for (interval, idata) in iter {
            starts.push(interval.start());
            ends.push(interval.end());
            data.push(idata);
            max_len = max_len.max(interval.len());
        }

        Self {
            data,
            starts,
            ends,
            max_len,
        }
    }

    #[inline]
    fn lower_bound(&self, start: Idx) -> usize {
        // We need to find the first element that `might` overlap with the query interval.
        // This is identical to the first element where start - max length > query start.
        let boundary = start.saturating_sub(self.max_len);
        self.starts.binary_search(&boundary).unwrap_or_else(|x| x)
    }

    /// Creates an iterator over entries overlapping the given interval.
    ///
    /// This is the primary low-level query mechanism used by the `ITree` trait methods.
    ///
    /// # Arguments
    ///
    /// * `interval`: The interval to query for overlapping entries.
    ///
    /// # Returns
    ///
    /// An iterator yielding `(Interval<Idx>, &'tree Data)` tuples for overlapping entries.
    #[inline]
    pub fn query(&self, interval: Interval<Idx>) -> Iter<Idx, Data> {
        let query_cursor = QueryCursor {
            query: interval,
            cursor: self.lower_bound(interval.start()),
        };

        Iter {
            cursor: query_cursor,
            bits: self,
        }
    }

    /// Creates a mutable iterator over entries overlapping the given interval.
    ///
    /// This is the primary low-level query mechanism.
    ///
    /// # Arguments
    ///
    /// * `interval`: The interval to query for overlapping entries.
    ///
    /// # Returns
    ///
    /// An iterator yielding `(Interval<Idx>, &'tree mut Data)` tuples for overlapping entries.
    pub fn query_mut(&mut self, interval: Interval<Idx>) -> IterMut<Idx, Data> {
        let query_cursor = QueryCursor {
            query: interval,
            cursor: self.lower_bound(interval.start()),
        };

        IterMut {
            cursor: query_cursor,
            bits: self,
        }
    }

    /// Returns the number of intervals stored in the tree.
    #[inline]
    pub fn len(&self) -> usize {
        self.starts.len()
    }

    /// Returns `true` if the tree contains no intervals.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.starts.is_empty()
    }

    /// Provides access to the raw data elements stored in the tree.
    /// The order corresponds to the intervals sorted by start position.
    pub fn data(&self) -> &[Data] {
        &self.data
    }

    /// Provides mutable access to the raw data elements stored in the tree.
    /// The order corresponds to the intervals sorted by start position.
    pub fn data_mut(&mut self) -> &mut [Data] {
        &mut self.data
    }

    /// Returns a builder for constructing a new `Bits` interval tree.
    pub fn builder() -> BitsBuilder<Idx, Data> {
        BitsBuilder::default()
    }
}

/// Helper struct to manage the state of an ongoing query.
struct QueryCursor<Idx: PrimInt> {
    /// The query interval.
    query: Interval<Idx>,
    /// The current position in the Bits tree.
    /// The cursor might be behind the target interval, but never ahead.
    cursor: usize,
}

impl<Idx: PrimInt> QueryCursor<Idx> {
    /// Advances the cursor to the next overlapping interval. Returns `None` if no more
    /// overlapping intervals are found.
    #[inline]
    fn next<Data>(&mut self, bits: &Bits<Idx, Data>) -> Option<(Interval<Idx>, usize)> {
        // Fast-forward to the first element that starts after the query interval.
        while self.cursor < bits.len() && bits.ends[self.cursor] <= self.query.start() {
            self.cursor += 1;
        }
        // If we've reached the end or the next element starts after the query interval, we're done.
        if self.cursor >= bits.len() || bits.starts[self.cursor] >= self.query.end() {
            return None;
        }

        // Otherwise, we must have an overlap.
        let segment = unsafe {
            Interval::new(bits.starts[self.cursor], bits.ends[self.cursor]).unwrap_unchecked()
        };
        debug_assert!(segment.intersects(&self.query));
        let result = Some((segment, self.cursor));
        self.cursor += 1;
        result
    }
}

/// An iterator over overlapping intervals and data references produced by `Bits::query`.
pub struct Iter<'tree, Idx: PrimInt, Data> {
    cursor: QueryCursor<Idx>,
    bits: &'tree Bits<Idx, Data>,
}

impl<'tree, Idx: PrimInt, Data> Iterator for Iter<'tree, Idx, Data> {
    type Item = (Interval<Idx>, &'tree Data);

    fn next(&mut self) -> Option<Self::Item> {
        let (segment, cursor) = self.cursor.next(self.bits)?;
        Some((segment, &self.bits.data[cursor]))
    }
}

/// An iterator over overlapping intervals and data references produced by `Bits::query`.
/// This iterator allows for mutable access to the data elements.
pub struct IterMut<'tree, Idx: PrimInt, Data> {
    cursor: QueryCursor<Idx>,
    bits: &'tree mut Bits<Idx, Data>,
}

impl<'tree, Idx: PrimInt, Data> Iterator for IterMut<'tree, Idx, Data> {
    type Item = (Interval<Idx>, &'tree mut Data);

    fn next(&mut self) -> Option<Self::Item> {
        let (segment, cursor) = self.cursor.next(self.bits)?;
        // SAFETY: The cursor is guaranteed to be within bounds of the data vector. And the 'tree
        // lifetime is at least as long as the '_ lifetime of the IterMut borrow.
        unsafe {
            let element = self.bits.data.get_mut(cursor).unwrap_unchecked() as *mut Data;
            Some((segment, element.as_mut().unwrap_unchecked()))
        }
    }
}

impl<Idx: PrimInt, Data> ITree for Bits<Idx, Data> {
    type Idx = Idx;
    type Data = Data;

    fn data(&self) -> impl Iterator<Item = &Self::Data> {
        self.data.iter()
    }

    fn intervals(&self) -> impl Iterator<Item = Interval<Self::Idx>> {
        self.starts
            .iter()
            .zip(self.ends.iter())
            // SAFETY: Interval validity (start < end) is ensured by constructor logic.
            .map(|(x, y)| unsafe { Interval::new(*x, *y).unwrap_unchecked() })
    }

    fn records(&self) -> impl Iterator<Item = (Interval<Self::Idx>, &Self::Data)> {
        self.intervals().zip(self.data.iter())
    }

    fn intersect_interval<'hits, 'tree: 'hits>(
        &'tree self,
        interval: &Interval<Self::Idx>,
        buffer: &mut Hits<'hits, Self::Idx, Self::Data>,
    ) {
        buffer.clear();

        let mut query = self.query(*interval);
        while let Some((interval, element)) = query.next() {
            buffer.push(interval, element)
        }
    }

    fn batch_intersect_intervals<'hits, 'tree: 'hits>(
        &'tree self,
        intervals: &[Interval<Self::Idx>],
        buffer: &mut BatchHits<'hits, Self::Idx, Self::Data>,
    ) {
        buffer.clear();

        for query in intervals {
            let mut query = self.query(*query);
            let mut hits = buffer.add_hits();
            while let Some((interval, element)) = query.next() {
                hits.add(interval, element);
            }
            hits.push();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval_tree::test_stand;

    #[test]
    fn test_bits_interval_tree() {
        test_stand::run_all(Bits::builder());
    }
}
