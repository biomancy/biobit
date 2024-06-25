use std::collections::{BTreeSet, HashSet};

use by_address::ByThinAddress;
use derive_getters::Dissolve;

use biobit_core_rs::loc::{Contig, Interval, LikeInterval, Locus};
use biobit_core_rs::num::PrimInt;

#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct OverlapSteps<'a, Idx: PrimInt, T: ?Sized> {
    cache: BTreeSet<Idx>,
    boundaries: Vec<Idx>,
    annotation: Vec<HashSet<ByThinAddress<&'a T>>>,
}

impl<'a, Idx: PrimInt, T: ?Sized> OverlapSteps<'a, Idx, T> {
    pub const DEFAULT_CAPACITY: usize = 64;
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: BTreeSet::new(),
            boundaries: Vec::with_capacity(capacity),
            annotation: Vec::with_capacity(capacity),
        }
    }
    pub fn new() -> Self { Self::with_capacity(Self::DEFAULT_CAPACITY) }

    pub fn build<Ctg: Contig>(&mut self, query: &OverlapIntervals<'a, Ctg, Idx, T>) {
        let locus = query.locus();

        self.cache.clear();
        self.cache.insert(locus.interval().start());
        self.cache.insert(locus.interval().end());
        for it in query.intervals().iter() {
            self.cache.insert(it.start());
            self.cache.insert(it.end());
        }

        self.boundaries.clear();
        self.boundaries.extend(self.cache.iter());

        // Clear existing references and allocate enough space for all the annotations if needed
        for anvec in self.annotation.iter_mut() {
            anvec.clear();
        }
        if self.annotation.len() < self.boundaries.len() - 1 {
            self.annotation.resize(
                self.boundaries.len() - 1,
                HashSet::with_capacity(Self::DEFAULT_CAPACITY),
            );
        }

        // Create stepped annotations
        for (it, anno) in query.iter() {
            let st = self.boundaries.binary_search(&it.start()).unwrap();
            let en = self.boundaries.binary_search(&it.end()).unwrap();
            for stanno in self.annotation[st..en].iter_mut() {
                stanno.insert(ByThinAddress(anno));
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(Idx, Idx, &HashSet<ByThinAddress<&'a T>>)> {
        // Note - annotation might be longer than boundaries and contain unspecified sets at the end
        // It's done to avoid reallocation of the hash sets
        (0..self.boundaries.len() - 1).map(|x| (self.boundaries[x], self.boundaries[x + 1], &self.annotation[x]))
    }


    pub fn len(&self) -> usize { self.annotation.len() }
}

impl<'a, Idx: PrimInt, T> Default for OverlapSteps<'a, Idx, T> {
    fn default() -> Self { Self::new() }
}


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Dissolve)]
pub struct OverlapIntervals<'a, Ctg: Contig, Idx: PrimInt, T: ?Sized> {
    locus: Locus<Ctg, Idx>,
    intervals: Vec<Interval<Idx>>,
    annotations: Vec<&'a T>,
}


impl<'a, Ctg: Contig, Idx: PrimInt, T> Default for OverlapIntervals<'a, Ctg, Idx, T> {
    fn default() -> Self { Self::new() }
}


impl<'a, Ctg: Contig, Idx: PrimInt, T: ?Sized> OverlapIntervals<'a, Ctg, Idx, T> {
    pub const DEFAULT_CAPACITY: usize = 32;

    pub fn new() -> Self { Self::with_capacity(Self::DEFAULT_CAPACITY) }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            locus: Locus::default(),
            intervals: Vec::with_capacity(capacity),
            annotations: Vec::with_capacity(capacity),
        }
    }

    pub fn reset<'b>(
        mut self,
        locus: Locus<Ctg, Idx>,
        data: impl Iterator<Item=(Interval<Idx>, &'b T)>,
    ) -> OverlapIntervals<'b, Ctg, Idx, T> {
        let (_, mut _intervals, mut _annotations) = self.dissolve();
        _intervals.clear();
        // https://github.com/rust-lang/rfcs/pull/2802#issuecomment-871512348
        _annotations.clear();
        let mut _annotations: Vec<&'b T> = _annotations.into_iter().map(|_| unreachable!()).collect();

        for (it, anno) in data {
            _intervals.push(it);
            _annotations.push(anno);
        }

        OverlapIntervals { locus, intervals: _intervals, annotations: _annotations }
    }

    pub fn locus(&self) -> &Locus<Ctg, Idx> { &self.locus }
    pub fn intervals(&self) -> &[Interval<Idx>] { &self.intervals }
    pub fn annotations(&self) -> &[&'a T] { &self.annotations }

    pub fn iter(&self) -> impl Iterator<Item=(&'_ Interval<Idx>, &'a T)> {
        self.intervals.iter().zip(self.annotations.iter().map(|&x| x))
    }

    pub fn len(&self) -> usize { self.intervals.len() }
}

#[cfg(test)]
mod tests {
    use biobit_core_rs::loc::Orientation;

    use super::*;

    #[test]
    fn test_overlap_query() {
        let locus = Locus::new("chr1".to_string(), (0..10).try_into().unwrap(), Orientation::Dual);
        let data = vec![
            ((1..3).try_into().unwrap(), "a"),
            ((4..6).try_into().unwrap(), "b"),
            ((7..9).try_into().unwrap(), "c"),
        ];

        let mut overlap = OverlapIntervals::new();
        overlap = overlap.reset(locus, data.into_iter());

        assert_eq!(overlap.len(), 3);

        let mut iter = overlap.iter();
        for (it, anno) in [
            ((1..3).try_into().unwrap(), "a"),
            ((4..6).try_into().unwrap(), "b"),
            ((7..9).try_into().unwrap(), "c"),
        ].iter() {
            assert_eq!(iter.next(), Some((it, *anno)));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_overlap_to_steps_1() {
        let locus = Locus::new("chr1".to_string(), (0..10).try_into().unwrap(), Orientation::Dual);
        let data = vec![
            ((4..6).try_into().unwrap(), "b"),
            ((7..9).try_into().unwrap(), "c"),
            ((1..3).try_into().unwrap(), "a"),
        ];

        let mut query = OverlapIntervals::new();
        query = query.reset(locus, data.into_iter());

        let mut steps = OverlapSteps::new();
        steps.build(&query);
        assert_eq!(steps.len(), 7);

        let mut iter = steps.iter();
        for (st, en, anno) in [
            (0, 1, &HashSet::new()),
            (1, 3, &HashSet::from_iter(vec![ByThinAddress("a")])),
            (3, 4, &HashSet::from_iter(vec![])),
            (4, 6, &HashSet::from_iter(vec![ByThinAddress("b")])),
            (6, 7, &HashSet::from_iter(vec![])),
            (7, 9, &HashSet::from_iter(vec![ByThinAddress("c")])),
            (9, 10, &HashSet::new()),
        ].iter() {
            assert_eq!(iter.next(), Some((*st, *en, *anno)));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_overlap_to_steps_2() {
        let locus = Locus::new("chr1".to_string(), (0..10).try_into().unwrap(), Orientation::default());
        let data = vec![
            ((0..2).try_into().unwrap(), "a"),
            ((8..10).try_into().unwrap(), "e"),
            ((2..4).try_into().unwrap(), "b"),
            ((4..6).try_into().unwrap(), "c"),
            ((6..8).try_into().unwrap(), "d"),
        ];

        let mut query = OverlapIntervals::new();
        query = query.reset(locus, data.into_iter());

        let mut steps = OverlapSteps::new();
        steps.build(&query);
        assert_eq!(steps.len(), 5);

        let mut iter = steps.iter();
        for (st, en, anno) in [
            (0, 2, &HashSet::from_iter(vec![ByThinAddress("a")])),
            (2, 4, &HashSet::from_iter(vec![ByThinAddress("b")])),
            (4, 6, &HashSet::from_iter(vec![ByThinAddress("c")])),
            (6, 8, &HashSet::from_iter(vec![ByThinAddress("d")])),
            (8, 10, &HashSet::from_iter(vec![ByThinAddress("e")])),
        ].iter() {
            assert_eq!(iter.next(), Some((*st, *en, *anno)));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_overlap_to_steps_nested_intervals() {
        let locus = Locus::new("chr1".to_string(), (0..10).try_into().unwrap(), Orientation::Dual);
        let data = vec![
            ((1..9).try_into().unwrap(), "a"),
            ((2..8).try_into().unwrap(), "b"),
            ((3..7).try_into().unwrap(), "c"),
        ];

        let mut query = OverlapIntervals::new();
        query = query.reset(locus, data.into_iter());

        let mut steps = OverlapSteps::new();
        steps.build(&query);
        assert_eq!(steps.len(), 7);

        let mut iter = steps.iter();
        for (st, en, anno) in [
            (0, 1, &HashSet::new()),
            (1, 2, &HashSet::from_iter(vec![ByThinAddress("a")])),
            (2, 3, &HashSet::from_iter(vec![ByThinAddress("a"), ByThinAddress("b")])),
            (3, 7, &HashSet::from_iter(vec![ByThinAddress("a"), ByThinAddress("b"), ByThinAddress("c")])),
            (7, 8, &HashSet::from_iter(vec![ByThinAddress("a"), ByThinAddress("b")])),
            (8, 9, &HashSet::from_iter(vec![ByThinAddress("a")])),
            (9, 10, &HashSet::new()),
        ].iter() {
            assert_eq!(iter.next(), Some((*st, *en, *anno)));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_overlap_to_steps_empty_intervals() {
        let locus = Locus::new("chr1".to_string(), (0..10).try_into().unwrap(), Orientation::Dual);

        let mut query = OverlapIntervals::new();
        query = query.reset(locus, std::iter::empty());

        let mut steps: OverlapSteps<'_, usize, usize> = OverlapSteps::new();
        steps.build(&query);
        assert_eq!(steps.len(), 1);

        let mut iter = steps.iter();
        assert_eq!(iter.next(), Some((0, 10, &HashSet::new())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_overlap_to_steps_single_interval() {
        let locus = Locus::new("chr1".to_string(), (0..10).try_into().unwrap(), Orientation::Dual);
        let data = vec![((1..9).try_into().unwrap(), "a")];

        let mut query = OverlapIntervals::new();
        query = query.reset(locus, data.into_iter());

        let mut steps = OverlapSteps::new();
        steps.build(&query);
        assert_eq!(steps.len(), 3);

        let mut iter = steps.iter();
        for (st, en, anno) in [
            (0, 1, &HashSet::new()),
            (1, 9, &HashSet::from_iter(vec![ByThinAddress("a")])),
            (9, 10, &HashSet::new()),
        ].iter() {
            assert_eq!(iter.next(), Some((*st, *en, *anno)));
        }
        assert_eq!(iter.next(), None);
    }
}
