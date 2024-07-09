use ::higher_kinded_types::prelude::*;
use impl_tools::autoimpl;

#[autoimpl(for < T: trait + ? Sized > & mut T, Box < T >)]
pub trait LendingIterator {
    type Item: ForLifetime;

    fn next(&mut self) -> Option<<Self::Item as ForLifetime>::Of<'_>>;
}
