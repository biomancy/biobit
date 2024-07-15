use ::higher_kinded_types::ForLt;
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use eyre::Result;

use super::super::{core::Core, source::Source};
use super::transform::Transform;

#[derive(PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Transformed<S: Source, T: Transform<S::Iter, InItem = S::Item>> {
    source: S,
    transform: T,
    args: T::Args,
    cache: T::Cache,
}

impl<S: Source, T: Transform<S::Iter, InItem = S::Item>> Clone for Transformed<S, T> {
    fn clone(&self) -> Self {
        Self {
            source: dyn_clone::clone(&self.source),
            transform: dyn_clone::clone(&self.transform),
            args: dyn_clone::clone(&self.args),
            cache: dyn_clone::clone(&self.cache),
        }
    }
}

impl<S, T> Core for Transformed<S, T>
where
    S: Source,
    T: Transform<S::Iter, InItem = S::Item>,
{
    type Args = S::Args;
    type Item = T::OutItem;

    fn batch_size(&self) -> usize {
        self.source.batch_size()
    }

    fn with_batch_size(&mut self, batch_size: usize) {
        self.source.with_batch_size(batch_size);
    }
}

impl<S, T> Source for Transformed<S, T>
where
    S: Source,
    T: Transform<S::Iter, InItem = S::Item>,
{
    type Iter = T::OutIter;

    fn fetch<'borrow>(
        &'borrow mut self,
        args: <<Self as Core>::Args as ForLt>::Of<'_>,
    ) -> Result<<Self::Iter as ForLt>::Of<'borrow>> {
        self.transform.setup(self.batch_size(), &mut self.cache);
        let iter = self.source.fetch(args)?;

        Ok(self.transform.transform(iter, &self.args, &mut self.cache))
    }
}

// #[derive(Debug, Constructor, Dissolve)]
// pub struct TransformIterator<'a, S: LendingIterator<Item = T::InItem>, T: Transform<S::Iter>> {
//     source_iterator: S,
//     transform: &'a mut T,
//     args: &'a T::Args,
//     cache: &'a mut T::Cache,
// }
//
// impl<'a, S: LendingIterator<Item = T::InItem>, T: Transform> LendingIterator
//     for TransformIterator<'a, S, T>
// {
//     type Item = T::OutItem;
//
//     fn next(&mut self) -> Option<<Self::Item as ForLifetime>::Of<'_>> {
//         // self.source_iterator
//         //     .next()
//         //     .map(|item| self.transform.transform(&item, self.args, self.cache))
//         todo!()
//     }
// }
