use crate::scalar;
use crate::store::Unpack;

macro_rules! impl_unpack_to {
    ($array_store:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl Unpack<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn unpack_unchecked<ScalarLayout>(
                    &self,
                    scalars: &mut [$scalar_store],
                )
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get() as u32;
                    let mut buffer = *self;

                    for scalar in scalars {
                        buffer = buffer.rotate_left(width);
                        *scalar = (buffer as $scalar_store) & ScalarLayout::MASK;
                    }
                }
            }
        )+
    };
}

impl_unpack_to!(u8 => u8);
impl_unpack_to!(u16 => u8, u16);
impl_unpack_to!(u32 => u8, u16, u32);
impl_unpack_to!(u64 => u8, u16, u32, u64);
