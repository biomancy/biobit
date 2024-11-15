use dyn_clone::DynClone;
use eyre::Result;
use higher_kinded_types::prelude::*;
use impl_tools::autoimpl;

use crate::source::dyn_source::DynSource;
use crate::LendingIterator;

use super::{
    core::{AnyMap, Core},
    transform::{Transform, Transformed},
};

#[autoimpl(for<T: trait + ?Sized> Box<T> where Box<T>: Clone)]
pub trait Source: Core + DynClone + Send + Sync {
    type Iter: for<'borrow> ForLt<Of<'borrow>: LendingIterator<Item = Self::Item>>;

    #[allow(clippy::needless_lifetimes)]
    fn fetch<'borrow, 'args>(
        &'borrow mut self,
        args: <<Self as Core>::Args as ForLt>::Of<'args>,
    ) -> Result<<Self::Iter as ForLt>::Of<'borrow>>;

    fn with_transform<T: Transform<Self::Iter, InItem = Self::Item>>(
        self,
        transform: T,
        args: T::Args,
    ) -> Transformed<Self, T>
    where
        Self: Sized,
    {
        Transformed::new(self, transform, args)
    }

    fn to_dynsrc(self) -> DynSourceBridge<Self>
    where
        Self: Sized,
    {
        DynSourceBridge { slf: self }
    }

    fn boxed(self) -> Box<dyn Source<Args = Self::Args, Item = Self::Item, Iter = Self::Iter>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

dyn_clone::clone_trait_object!(<Args, Item, Iter> Source<Args=Args, Item=Item, Iter=Iter>);

#[derive(Debug, PartialEq, Eq, Hash, Default, Copy, PartialOrd, Ord)]
pub struct DynSourceBridge<S: Source> {
    slf: S,
}

impl<S: Source> Clone for DynSourceBridge<S> {
    fn clone(&self) -> Self {
        Self {
            slf: dyn_clone::clone(&self.slf),
        }
    }
}

impl<S: Source> Core for DynSourceBridge<S> {
    type Args = S::Args;
    type Item = S::Item;

    fn populate_caches(&mut self, cache: &mut AnyMap) {
        self.slf.populate_caches(cache)
    }

    fn release_caches(&mut self, cache: &mut AnyMap) {
        self.slf.release_caches(cache)
    }

    #[inline(always)]
    fn batch_size(&self) -> usize {
        self.slf.batch_size()
    }

    #[inline(always)]
    fn with_batch_size(&mut self, batch_size: usize) {
        self.slf.with_batch_size(batch_size)
    }
}

impl<S: Source> DynSource for DynSourceBridge<S> {
    #[inline(always)]
    fn fetch<'borrow, 'args>(
        &'borrow mut self,
        args: <<Self as Core>::Args as ForLt>::Of<'args>,
    ) -> Result<Box<dyn 'borrow + LendingIterator<Item = Self::Item>>> {
        Ok(Box::new(Source::fetch(&mut self.slf, args)?))
    }
}
