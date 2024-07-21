use ::dyn_clone::DynClone;
use ::higher_kinded_types::ForLt;

use crate::LendingIterator;

use super::super::core::AnyMap;

pub trait Transform<InIter>: Clone + Send + Sync
where
    InIter: for<'borrow> ForLt<Of<'borrow>: LendingIterator<Item = Self::InItem>>,
{
    type Args: DynClone + Send + Sync;

    type OutIter: for<'borrow> ForLt<Of<'borrow>: LendingIterator<Item = Self::OutItem>>;

    type InItem: ForLt;

    type OutItem: ForLt;

    fn populate_caches(&mut self, cache: &mut AnyMap);

    fn release_caches(&mut self, cache: &mut AnyMap);

    fn batch_size(&self) -> usize;

    fn with_batch_size(&mut self, batch_size: usize);

    fn transform<'borrow, 'args>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        args: &'args Self::Args,
    ) -> <Self::OutIter as ForLt>::Of<'borrow>;
}
