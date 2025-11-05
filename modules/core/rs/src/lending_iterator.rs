use std::marker::PhantomData;

use ::impl_tools::autoimpl;
use derive_getters::Dissolve;
use derive_more::Constructor;
use higher_kinded_types::ForFixed;
use higher_kinded_types::prelude::*;

#[autoimpl(for <T: trait + ?Sized> &mut T, Box <T>)]
pub trait LendingIterator {
    type Item: ForLt;

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>>;

    fn map<Fun, Out>(self, f: Fun) -> Map<Self, Fun, Out>
    where
        Self: Sized,
        Out: ForLt,
        for<'iter> Fun:
            FnMut(&'iter (), <Self::Item as ForLt>::Of<'iter>) -> <Out as ForLt>::Of<'iter>,
    {
        Map::new(self, f, PhantomData)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Dissolve, Constructor)]
pub struct Map<I, Fun, Out> {
    iter: I,
    map: Fun,
    _phantom: PhantomData<Out>,
}

impl<I, Fun, Out> LendingIterator for Map<I, Fun, Out>
where
    I: LendingIterator,
    Out: ForLt,
    for<'iter> Fun: FnMut(
        &'iter (),
        <<I as LendingIterator>::Item as ForLt>::Of<'iter>,
    ) -> <Out as ForLt>::Of<'iter>,
{
    type Item = Out;

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.iter.next() {
            None => None,
            Some(item) => Some((self.map)(&(), item)),
        }
    }
}

pub trait IntoLendingIterator {
    type Item: ForLt;
    fn into_lending(self) -> impl LendingIterator<Item = Self::Item>;
}

pub struct IteratorAdapter<I: Iterator> {
    pub lended: I,
}

impl<I: Iterator> LendingIterator for IteratorAdapter<I> {
    type Item = ForFixed<I::Item>;

    fn next(&mut self) -> Option<<Self::Item as ForLifetime>::Of<'_>> {
        self.lended.next()
    }
}

impl<T: Iterator> IntoLendingIterator for T {
    type Item = ForFixed<T::Item>;

    fn into_lending(self) -> impl LendingIterator<Item = Self::Item> {
        IteratorAdapter { lended: self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map() {
        let mut iter = (0..10usize)
            .into_lending()
            .map::<_, ForFixed<usize>>(|_, x| x * 2);
        for i in 0..10 {
            assert_eq!(iter.next(), Some(i * 2));
        }
        assert_eq!(iter.next(), None);
    }
}
