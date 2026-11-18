use crate::scalar;
use crate::store::{ScalarSet, StaticStore};

macro_rules! impl_scalar_set {
    ($array_store:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl ScalarSet<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn set_unchecked<ScalarLayout>(&mut self, pos: usize, scalar: $scalar_store)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let shift = <$array_store as StaticStore>::CAPACITY_BITS.get() - ((pos + 1) * width);

                    // Move the scalar to the correct position
                    let scalar = (scalar as $array_store) << shift;

                    // Mask out any existing bits at the target position
                    let mask = (ScalarLayout::MASK as $array_store) << shift;

                    // Replace the masked bits with the new scalar
                    *self = (*self & !mask) | scalar;
                }
            }
        )+
    };
}

impl_scalar_set!(u8 => u8);
impl_scalar_set!(u16 => u8, u16);
impl_scalar_set!(u32 => u8, u16, u32);
impl_scalar_set!(u64 => u8, u16, u32, u64);
