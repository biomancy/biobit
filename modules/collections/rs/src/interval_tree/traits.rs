use std::fmt::Debug;

use gat_lending_iterator::LendingIterator;

use biobit_core_rs::loc::{Interval, LikeInterval};
use biobit_core_rs::num::PrimInt;

pub trait IntervalTreeBuilder {
    type Idx: PrimInt;
    type Value;
    type Tree: IntervalTree;

    fn add(self, interval: &impl LikeInterval<Idx=Self::Idx>, value: Self::Value) -> Self;
    fn build(self) -> Self::Tree;
}

pub trait IntervalTreeElement<'iter, 'borrow>
{
    type Idx: PrimInt + 'iter;
    type Value: 'borrow;
    fn interval(&self) -> &'iter Interval<Self::Idx>;
    fn value(&self) -> &'borrow Self::Value;
}

pub trait IntervalTreeLendingIterator<'borrow, Idx: PrimInt, Value> {
    type Item<'iter>: IntervalTreeElement<'iter, 'borrow, Idx=Idx, Value=Value>
    where
        Self: 'iter;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}


pub trait IntervalTree
{
    type Idx: PrimInt;
    type Value;
    type Iter<'borrow>: IntervalTreeLendingIterator<'borrow, Self::Idx, Self::Value>
    where
        Self: 'borrow;
    fn intersection(&self, interval: &impl LikeInterval<Idx=Self::Idx>) -> Self::Iter<'_>;
}

impl<'a, 'b, Idx: PrimInt, Val> IntervalTreeElement<'a, 'b> for (&'a Interval<Idx>, &'b Val) {
    type Idx = Idx;
    type Value = Val;

    fn interval(&self) -> &'a Interval<Self::Idx> { self.0 }

    fn value(&self) -> &'b Self::Value { self.1 }
}


#[cfg(test)]
pub mod tests {
    use super::*;

    fn assert_iterator_eq<'a, Idx: PrimInt + Debug, T: PartialEq + Debug + 'a>(
        mut iter: impl IntervalTreeLendingIterator<'a, Idx, T>,
        expected: Vec<(Interval<Idx>, T)>,
    ) {
        for (interval, value) in expected {
            let elem = iter.next().unwrap();
            assert_eq!(elem.interval(), &interval);
            assert_eq!(elem.value(), &value);
        }
        assert!(iter.next().is_none());
    }

    pub fn test_empty_tree<T>(builder: T)
    where
        T: IntervalTreeBuilder<Idx=usize, Value=usize>,
        <T as IntervalTreeBuilder>::Tree: IntervalTree<Idx=usize, Value=usize>,
    {
        let tree = builder.build();
        assert_iterator_eq::<usize, usize>(
            tree.intersection(&Interval::new(5, 15).unwrap()),
            vec![],
        );
    }

    pub fn test_single_interval_tree<T>(mut builder: T)
    where
        T: IntervalTreeBuilder<Idx=usize, Value=usize>,
        <T as IntervalTreeBuilder>::Tree: IntervalTree<Idx=usize, Value=usize>,
    {
        let tree = builder
            .add(&Interval::new(10, 20).unwrap(), 1)
            .build();


        // Off-range queries
        assert_iterator_eq::<usize, usize>(
            tree.intersection(&Interval::new(5, 9).unwrap()),
            vec![],
        );
        assert_iterator_eq::<usize, usize>(
            tree.intersection(&Interval::new(21, 25).unwrap()),
            vec![],
        );

        // Touching query
        assert_iterator_eq::<usize, usize>(
            tree.intersection(&Interval::new(0, 10).unwrap()),
            vec![],
        );

        // Intersecting queries
        for interval in [
            Interval::new(5, 15).unwrap(),
            Interval::new(15, 25).unwrap(),
            Interval::new(5, 25).unwrap(),
        ] {
            assert_iterator_eq::<usize, usize>(
                tree.intersection(&interval),
                vec![(Interval::new(10, 20).unwrap(), 1)],
            );
        }
    }

    pub fn test_multi_interval_tree<T>(mut builder: T)
    where
        T: IntervalTreeBuilder<Idx=usize, Value=usize>,
        <T as IntervalTreeBuilder>::Tree: IntervalTree<Idx=usize, Value=usize>,
    {
        let tree = builder
            .add(&Interval::new(1, 10).unwrap(), 1)
            .add(&Interval::new(5, 15).unwrap(), 2)
            .add(&Interval::new(10, 20).unwrap(), 3)
            .build();
        let interval = Interval::new(5, 15).unwrap();

        let mut iter = tree.intersection(&interval);
        for (interval, value) in [
            (Interval::new(1usize, 10).unwrap(), 1usize),
            (Interval::new(5, 15).unwrap(), 2),
            (Interval::new(10, 20).unwrap(), 3),
        ] {
            let elem = iter.next().unwrap();
            assert_eq!(elem.interval(), &interval);
            assert_eq!(elem.value(), &value);
        }
        assert!(iter.next().is_none());
    }
}
