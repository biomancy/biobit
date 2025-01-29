use super::traits::{Builder, ITree};
use crate::interval_tree::overlap::Elements;
use biobit_core_rs::{
    loc::{Interval, IntervalOp},
    num::PrimInt,
};
use derive_getters::Dissolve;
use derive_more::From;
use itertools::Itertools;
// https://doi.org/10.1093/bioinformatics/bts652

#[derive(Debug, Clone, From, Dissolve)]
pub struct BitsBuilder<Idx: PrimInt, Element> {
    data: Vec<(Element, Interval<Idx>)>,
}

impl<Idx: PrimInt, Element> Default for BitsBuilder<Idx, Element> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<Idx: PrimInt + 'static, Element: Clone> Builder for BitsBuilder<Idx, Element> {
    type Target = Bits<Idx, Element>;

    fn addi(
        mut self,
        interval: impl IntervalOp<Idx = <Self::Target as ITree>::Idx>,
        element: <Self::Target as ITree>::Element,
    ) -> Self {
        self.data.push((element, interval.as_interval()));
        self
    }

    fn add(
        mut self,
        data: impl Iterator<
            Item = (
                <Self::Target as ITree>::Element,
                impl IntervalOp<Idx = <Self::Target as ITree>::Idx>,
            ),
        >,
    ) -> Self {
        self.data
            .extend(data.map(|(element, interval)| (element, interval.as_interval())));
        self
    }

    fn build(self) -> Self::Target {
        Bits::new(self.data.into_iter())
    }
}

#[derive(Debug, Clone, Default, From, Dissolve)]
pub struct Bits<Idx: PrimInt, Element> {
    // Input elements and corresponding flat intervals, sorted by start position.
    elements: Vec<Element>,
    starts: Vec<Idx>,
    ends: Vec<Idx>,
    // Maximum length of all intervals for efficient queries.
    max_len: Idx,
}

impl<Idx: PrimInt, Element> Bits<Idx, Element> {
    pub fn new(data: impl Iterator<Item = (Element, Interval<Idx>)>) -> Self {
        let explen = data.size_hint().0;
        let mut starts = Vec::with_capacity(explen);
        let mut ends = Vec::with_capacity(explen);
        let mut elements = Vec::with_capacity(explen);
        let mut max_len = Idx::zero();

        for (data, interval) in data.sorted_by_key(|(_, interval)| *interval) {
            starts.push(interval.start());
            ends.push(interval.end());
            elements.push(data);
            max_len = max_len.max(interval.len());
        }

        Self {
            elements,
            starts,
            ends,
            max_len,
        }
    }

    fn lower_bound(&self, start: Idx) -> usize {
        // We need to find the first element that `might` overlap with the query interval.
        // This is identical to the first element where start - max length > query start.
        let boundary = start.saturating_sub(self.max_len);
        self.starts.binary_search(&boundary).unwrap_or_else(|x| x)
    }

    pub fn query(&self, start: Idx, end: Idx) -> Iter<Idx, Element> {
        let query_cursor = QueryCursor {
            start,
            end,
            cursor: self.lower_bound(start),
        };

        Iter {
            cursor: query_cursor,
            bits: self,
        }
    }

    pub fn query_mut(&mut self, start: Idx, end: Idx) -> IterMut<Idx, Element> {
        let query_cursor = QueryCursor {
            start,
            end,
            cursor: self.lower_bound(start),
        };

        IterMut {
            cursor: query_cursor,
            bits: self,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.starts.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.starts.is_empty()
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn builder() -> BitsBuilder<Idx, Element> {
        BitsBuilder::default()
    }
}

pub struct QueryCursor<Idx: PrimInt> {
    start: Idx,
    end: Idx,
    cursor: usize,
}

impl<Idx: PrimInt> QueryCursor<Idx> {
    pub fn next<Element>(&mut self, bits: &Bits<Idx, Element>) -> Option<(Interval<Idx>, usize)> {
        // Cursor might be behind the target interval, but never ahead.

        // Fast-forward to the first element that starts after the query interval.
        while self.cursor < bits.len() && bits.ends[self.cursor] <= self.start {
            self.cursor += 1;
        }
        // If we reached the end or the next element starts after the query interval, we are done.
        if self.cursor >= bits.len() || bits.starts[self.cursor] >= self.end {
            return None;
        }

        // Otherwise, we must have an overlap.
        let segment = unsafe {
            Interval::new(bits.starts[self.cursor], bits.ends[self.cursor]).unwrap_unchecked()
        };
        debug_assert!(segment.intersects(&Interval::new(self.start, self.end).unwrap()));
        let result = Some((segment, self.cursor));
        self.cursor += 1;
        result
    }
}

pub struct Iter<'borrow, Idx: PrimInt, Element> {
    cursor: QueryCursor<Idx>,
    bits: &'borrow Bits<Idx, Element>,
}

impl<'borrow, Idx: PrimInt, Element> Iterator for Iter<'borrow, Idx, Element> {
    type Item = (Interval<Idx>, &'borrow Element);

    fn next(&mut self) -> Option<Self::Item> {
        let (segment, cursor) = self.cursor.next(self.bits)?;
        Some((segment, &self.bits.elements[cursor]))
    }
}

pub struct IterMut<'borrow, Idx: PrimInt, Element> {
    cursor: QueryCursor<Idx>,
    bits: &'borrow mut Bits<Idx, Element>,
}

impl<'borrow, Idx: PrimInt, Element> Iterator for IterMut<'borrow, Idx, Element> {
    type Item = (Interval<Idx>, &'borrow mut Element);

    fn next(&mut self) -> Option<Self::Item> {
        let (segment, cursor) = self.cursor.next(self.bits)?;
        unsafe {
            let element = self.bits.elements.get_mut(cursor).unwrap_unchecked() as *mut Element;
            Some((segment, element.as_mut().unwrap_unchecked()))
        }
    }
}

impl<Idx: PrimInt, Element: Clone> ITree for Bits<Idx, Element> {
    type Idx = Idx;
    type Element = Element;

    fn overlap_single_element<'a>(
        &self,
        intervals: &[Interval<Self::Idx>],
        buffer: &'a mut Elements<Self::Idx, Self::Element>,
    ) -> &'a mut Elements<Self::Idx, Self::Element> {
        for interval in intervals {
            let mut adder = buffer.add();
            let mut query = self.query(interval.start(), interval.end());

            #[allow(clippy::while_let_on_iterator)]
            while let Some((interval, element)) = query.next() {
                adder.add(interval, element.clone());
            }
            adder.finish();
        }
        // debug_assert_eq!(buffer.len(), intervals.len());
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::super::traits::tests;
    use super::*;

    #[test]
    fn test_bits_interval_tree() {
        tests::test_interval_tree(Bits::builder());
    }
}
