use ::higher_kinded_types::prelude::*;
use dyn_clone::DynClone;
use eyre::Result;
use impl_tools::autoimpl;

use crate::LendingIterator;

use super::{core::Core, source::Source};

#[autoimpl(for<T: trait + ?Sized> Box<T> where Box<T>: Clone)]
pub trait DynSource: Core + DynClone + Send + Sync {
    fn fetch<'args, 'borrow>(
        &'borrow mut self,
        args: <Self::Args as ForLt>::Of<'args>,
    ) -> Result<Box<dyn 'borrow + LendingIterator<Item = Self::Item>>>;

    fn to_src(self) -> SourceBridge<Self>
    where
        Self: Sized,
    {
        SourceBridge { slf: self }
    }

    fn boxed(self) -> Box<dyn DynSource<Args = Self::Args, Item = Self::Item>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

dyn_clone::clone_trait_object!(<Args, Item> DynSource<Args=Args, Item=Item>);

#[derive(Debug, PartialEq, Eq, Hash, Default, Copy, PartialOrd, Ord)]
pub struct SourceBridge<S: DynSource> {
    slf: S,
}

impl<S: DynSource> Clone for SourceBridge<S> {
    fn clone(&self) -> Self {
        Self {
            slf: dyn_clone::clone(&self.slf),
        }
    }
}

impl<S: DynSource> Core for SourceBridge<S> {
    type Args = S::Args;
    type Item = S::Item;
    #[inline(always)]
    fn batch_size(&self) -> usize {
        self.slf.batch_size()
    }

    #[inline(always)]
    fn with_batch_size(&mut self, batch_size: usize) {
        self.slf.with_batch_size(batch_size)
    }
}

impl<S: DynSource> Source for SourceBridge<S> {
    type Iter = For!(<'borrow> = Box<dyn 'borrow + LendingIterator<Item = Self::Item>>);

    #[inline(always)]
    fn fetch<'args, 'borrow>(
        &'borrow mut self,
        args: <<Self as Core>::Args as ForLt>::Of<'args>,
    ) -> Result<Box<dyn 'borrow + LendingIterator<Item = Self::Item>>> {
        Ok(Box::new(DynSource::fetch(&mut self.slf, args)?))
    }
}
