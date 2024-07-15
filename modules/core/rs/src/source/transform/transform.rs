use ::dyn_clone::DynClone;
use ::higher_kinded_types::ForLt;

use crate::LendingIterator;

pub trait Transform<InIter>: Clone + Send + Sync
where
    InIter: for<'borrow> ForLt<Of<'borrow>: LendingIterator<Item = Self::InItem>>,
{
    type Args: DynClone + Send + Sync;

    type Cache: DynClone + Default + Send + Sync;

    type OutIter: for<'borrow> ForLt<Of<'borrow>: LendingIterator<Item = Self::OutItem>>;

    type InItem: ForLt;

    type OutItem: ForLt;

    fn setup(&mut self, batch_size: usize, cache: &mut Self::Cache);

    fn transform<'borrow>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        args: &'borrow Self::Args,
        cache: &'borrow mut Self::Cache,
    ) -> <Self::OutIter as ForLt>::Of<'borrow>;
}
