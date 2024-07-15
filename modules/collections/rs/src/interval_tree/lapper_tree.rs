use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
use derive_more::{Constructor, From};
use rust_lapper;

use biobit_core_rs::{
    LendingIterator,
    loc::{Segment, AsSegment},
    num::{PrimInt, Unsigned},
};

use super::traits::{Builder, ITree};

#[derive(Debug, Clone, From, Dissolve)]
pub struct LapperBuilder<Idx, T>(rust_lapper::Lapper<Idx, T>)
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync;

impl<Idx, T> Default for LapperBuilder<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    fn default() -> Self {
        rust_lapper::Lapper::new(vec![]).into()
    }
}

impl<Idx, T> LapperBuilder<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx, T> Builder for LapperBuilder<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync + 'static,
    T: Eq + Clone + Send + Sync + 'static,
{
    type Idx = Idx;
    type Value = T;
    type Tree = LapperTree<Idx, T>;

    fn add(mut self, interval: &impl AsSegment<Idx = Idx>, value: T) -> Self {
        let elem = rust_lapper::Interval {
            start: interval.start(),
            stop: interval.end(),
            val: value,
        };
        self.0.insert(elem);
        self
    }

    fn build(self) -> Self::Tree {
        LapperTree(self.0)
    }
}

#[derive(Debug, Clone, From, Dissolve)]
pub struct LapperTree<Idx, T>(rust_lapper::Lapper<Idx, T>)
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync,
    T: Eq + Clone + Send + Sync;

impl<Idx, T> ITree for LapperTree<Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync + 'static,
    T: Eq + Clone + Send + Sync + 'static,
{
    type Idx = Idx;
    type Value = T;
    type Iter = For!(<'borrow> = LapperIntersectionIter<'borrow, Idx, T>);

    fn intersection<'a>(
        &'a self,
        interval: &impl AsSegment<Idx = Self::Idx>,
    ) -> <Self::Iter as ForLt>::Of<'a> {
        let interval = Segment::new(interval.start(), interval.end())
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
    interval: Segment<Idx>,
    inner: rust_lapper::IterFind<'a, Idx, T>,
}

impl<'borrow, Idx: PrimInt, T> LendingIterator for LapperIntersectionIter<'borrow, Idx, T>
where
    Idx: PrimInt + Unsigned + Ord + Clone + Send + Sync + 'borrow,
    T: Eq + Clone + Send + Sync + 'borrow,
{
    type Item = For!(<'iter> = (&'iter Segment<Idx>, &'borrow T));

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.inner.next() {
            None => None,
            Some(item) => {
                self.interval = Segment::new(item.start, item.stop).unwrap();
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
        let builder: LapperBuilder<usize, usize> = rust_lapper::Lapper::new(vec![]).into();
        tests::test_empty_tree(builder.clone());
        tests::test_single_interval_tree(builder.clone());
        tests::test_multi_interval_tree(builder);
    }
}
