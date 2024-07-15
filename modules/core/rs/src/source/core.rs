use ::higher_kinded_types::prelude::*;
use impl_tools::autoimpl;

#[autoimpl(for < T: trait + ? Sized > &mut T, Box < T >)]
pub trait Core {
    type Args: ForLt;

    type Item: ForLt;

    fn batch_size(&self) -> usize;

    fn with_batch_size(&mut self, batch_size: usize);
}
