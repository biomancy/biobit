use derive_getters::Dissolve;
use derive_more::{Constructor, From};
use gat_lending_iterator::LendingIterator;
use rust_lapper;

use biobit_core_rs::loc::{Interval, LikeInterval};
use biobit_core_rs::num::{PrimInt, Unsigned};

use super::traits::{IntervalTree, IntervalTreeBuilder, IntervalTreeElement, IntervalTreeLendingIterator};

#[derive(Debug, Clone, From, Dissolve)]
pub struct LapperIntervalTreeBuilder<Idx, T>(rust_lapper::Lapper<Idx, T>)
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync;

impl<Idx, T> Default for LapperIntervalTreeBuilder<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    fn default() -> Self { rust_lapper::Lapper::new(vec![]).into() }
}

impl<Idx, T> LapperIntervalTreeBuilder<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    pub fn new() -> Self { Self::default() }
}

impl<Idx, T> IntervalTreeBuilder for LapperIntervalTreeBuilder<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    type Idx = Idx;
    type Value = T;
    type Tree = LapperIntervalTree<Idx, T>;

    fn add(mut self, interval: &impl LikeInterval<Idx=Idx>, value: T) -> Self {
        let elem = rust_lapper::Interval {
            start: interval.start(),
            stop: interval.end(),
            val: value,
        };
        self.0.insert(elem);
        self
    }

    fn build(self) -> Self::Tree { LapperIntervalTree(self.0) }
}

#[derive(Debug, Clone, From, Dissolve)]
pub struct LapperIntervalTree<Idx, T>(rust_lapper::Lapper<Idx, T>)
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync;

impl<Idx, T> IntervalTree for LapperIntervalTree<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    type Idx = Idx;
    type Value = T;
    type Iter<'borrow> = LapperIntersectionIter<'borrow, Idx, T>
    where
        Self: 'borrow;

    fn intersection(&self, interval: &impl LikeInterval<Idx=Self::Idx>) -> Self::Iter<'_> {
        let interval = Interval::new(interval.start(), interval.end())
            .expect("Invalid interval (lapper intersection)");

        LapperIntersectionIter::<Idx, T>::new(
            interval,
            self.0.find(interval.start(), interval.end()),
        )
    }
}

#[derive(Constructor)]
pub struct LapperIntersectionIter<'a, Idx: PrimInt, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    interval: Interval<Idx>,
    inner: rust_lapper::IterFind<'a, Idx, T>,
}

impl<'a, Idx: PrimInt, T> IntervalTreeLendingIterator<'a, Idx, T> for LapperIntersectionIter<'a, Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync + 'a,
    T: Eq + Clone + Send + Sync + 'a,
{
    type Item<'b> = (&'b Interval<Idx>, &'a T)
    where
        Self: 'b,
        Idx: 'b,
        T: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        match self.inner.next() {
            None => None,
            Some(item) => {
                self.interval.start = item.start;
                self.interval.end = item.stop;
                Some((&self.interval, &item.val))
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::traits::tests;

    #[test]
    fn test_lapper_interval_tree() {
        let builder: LapperIntervalTreeBuilder<usize, usize> = rust_lapper::Lapper::new(vec![]).into();
        tests::test_empty_tree(builder.clone());
        tests::test_single_interval_tree(builder.clone());
        tests::test_multi_interval_tree(builder);
    }
}