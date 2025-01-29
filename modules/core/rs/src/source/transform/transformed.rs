use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use eyre::Result;
use higher_kinded_types::ForLt;

use super::super::{
    core::{AnyMap, Core},
    source::Source,
};
use super::transform::Transform;

#[derive(PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Transformed<S: Source, T: Transform<S::Iter, InItem = S::Item>> {
    source: S,
    transform: T,
    args: T::Args,
}

impl<S: Source, T: Transform<S::Iter, InItem = S::Item>> Clone for Transformed<S, T> {
    fn clone(&self) -> Self {
        Self {
            source: dyn_clone::clone(&self.source),
            transform: dyn_clone::clone(&self.transform),
            args: dyn_clone::clone(&self.args),
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

    fn populate_caches(&mut self, cache: &mut AnyMap) {
        self.source.populate_caches(cache);
        self.transform.populate_caches(cache);
    }

    fn release_caches(&mut self, cache: &mut AnyMap) {
        self.source.release_caches(cache);
        self.transform.release_caches(cache);
    }

    fn batch_size(&self) -> usize {
        self.transform.batch_size()
    }

    fn with_batch_size(&mut self, batch_size: usize) {
        self.transform.with_batch_size(batch_size);
    }
}

impl<S, T> Source for Transformed<S, T>
where
    S: Source,
    T: Transform<S::Iter, InItem = S::Item>,
{
    type Iter = T::OutIter;

    #[allow(clippy::needless_lifetimes)]
    fn fetch<'borrow, 'args>(
        &'borrow mut self,
        args: <<Self as Core>::Args as ForLt>::Of<'args>,
    ) -> Result<<Self::Iter as ForLt>::Of<'borrow>> {
        let iter = self.source.fetch(args)?;
        Ok(self.transform.transform(iter, &self.args))
    }
}
