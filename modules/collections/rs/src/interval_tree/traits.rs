use ::higher_kinded_types::prelude::*;

use biobit_core_rs::{
    LendingIterator,
    loc::{Segment, AsSegment},
    num::PrimInt,
};

pub trait Builder {
    type Idx: PrimInt;
    type Value;
    type Tree: ITree;

    fn add(self, interval: &impl AsSegment<Idx = Self::Idx>, value: Self::Value) -> Self;
    fn build(self) -> Self::Tree;
}

pub trait TreeRecord<'borrow, 'iter> {
    type Idx: PrimInt + 'iter;
    type Value: 'borrow;
    fn interval(&self) -> &'iter Segment<Self::Idx>;
    fn value(&self) -> &'borrow Self::Value;
}

pub trait ITree {
    type Idx: PrimInt;
    type Value;
    type Iter: for<'borrow, 'iter> ForLt<
        Of<'borrow>: LendingIterator<
            Item: ForLt<
                Of<'iter>: TreeRecord<'borrow, 'iter, Idx = Self::Idx, Value = Self::Value>,
            >,
        >,
    >;

    fn intersection<'borrow>(
        &'borrow self,
        interval: &impl AsSegment<Idx = Self::Idx>,
    ) -> <Self::Iter as ForLt>::Of<'borrow>;
}

impl<'borrow, 'iter, Idx: PrimInt, Val> TreeRecord<'borrow, 'iter>
    for (&'iter Segment<Idx>, &'borrow Val)
{
    type Idx = Idx;
    type Value = Val;

    fn interval(&self) -> &'iter Segment<Self::Idx> {
        self.0
    }

    fn value(&self) -> &'borrow Self::Value {
        self.1
    }
}

#[cfg(test)]
pub mod tests {
    use std::fmt::Debug;

    use super::*;

    fn assert_iterator_eq<'borrow, Idx, T, Iter>(mut iter: Iter, expected: Vec<(Segment<Idx>, T)>)
    where
        Idx: PrimInt + Debug,
        T: PartialEq + Debug + 'borrow,
        Iter: LendingIterator + 'borrow,
        for<'iter> <<Iter as LendingIterator>::Item as ForLt>::Of<'iter>:
            TreeRecord<'borrow, 'iter, Idx = Idx, Value = T>,
    {
        for (interval, value) in expected {
            let elem = iter.next().unwrap();
            assert_eq!(elem.interval(), &interval);
            assert_eq!(elem.value(), &value);
        }
        assert!(iter.next().is_none());
    }

    pub fn test_empty_tree<T>(builder: T)
    where
        T: Builder<Idx = usize, Value = usize>,
        <T as Builder>::Tree: ITree<Idx = usize, Value = usize>,
    {
        let tree = builder.build();
        assert_iterator_eq::<usize, usize, _>(
            tree.intersection(&Segment::new(5, 15).unwrap()),
            vec![],
        );
    }

    pub fn test_single_interval_tree<T>(builder: T)
    where
        T: Builder<Idx = usize, Value = usize>,
        <T as Builder>::Tree: ITree<Idx = usize, Value = usize>,
    {
        let tree = builder.add(&Segment::new(10, 20).unwrap(), 1).build();

        // Off-range queries
        assert_iterator_eq::<usize, usize, _>(
            tree.intersection(&Segment::new(5, 9).unwrap()),
            vec![],
        );
        assert_iterator_eq::<usize, usize, _>(
            tree.intersection(&Segment::new(21, 25).unwrap()),
            vec![],
        );

        // Touching query
        assert_iterator_eq::<usize, usize, _>(
            tree.intersection(&Segment::new(0, 10).unwrap()),
            vec![],
        );

        // Intersecting queries
        for interval in [
            Segment::new(5, 15).unwrap(),
            Segment::new(15, 25).unwrap(),
            Segment::new(5, 25).unwrap(),
        ] {
            assert_iterator_eq::<usize, usize, _>(
                tree.intersection(&interval),
                vec![(Segment::new(10, 20).unwrap(), 1)],
            );
        }
    }

    pub fn test_multi_interval_tree<T>(builder: T)
    where
        T: Builder<Idx = usize, Value = usize>,
        <T as Builder>::Tree: ITree<Idx = usize, Value = usize>,
    {
        let tree = builder
            .add(&Segment::new(1, 10).unwrap(), 1)
            .add(&Segment::new(5, 15).unwrap(), 2)
            .add(&Segment::new(10, 20).unwrap(), 3)
            .build();
        let interval = Segment::new(5, 15).unwrap();

        let mut iter = tree.intersection(&interval);
        for (interval, value) in [
            (Segment::new(1usize, 10).unwrap(), 1usize),
            (Segment::new(5, 15).unwrap(), 2),
            (Segment::new(10, 20).unwrap(), 3),
        ] {
            let elem = iter.next().unwrap();
            assert_eq!(elem.interval(), &interval);
            assert_eq!(elem.value(), &value);
        }
        assert!(iter.next().is_none());
    }
}
