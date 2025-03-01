use std::any::Any;

use ::anymap3::Map;
use higher_kinded_types::prelude::*;
use impl_tools::autoimpl;

pub type AnyMap = Map<dyn Any + Send + Sync>;

#[autoimpl(for < T: trait + ? Sized > &mut T, Box < T >)]
pub trait Core {
    type Args: ForLt;

    type Item: ForLt;

    fn populate_caches(&mut self, cache: &mut AnyMap);

    fn release_caches(&mut self, cache: &mut AnyMap);

    fn batch_size(&self) -> usize;

    fn with_batch_size(&mut self, batch_size: usize);
}
