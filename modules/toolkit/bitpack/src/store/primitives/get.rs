use crate::scalar;
use crate::store::{ScalarGet, StaticStore};

macro_rules! impl_scalar_get {
    ($array_store:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl ScalarGet<$scalar_store> for $array_store
            {
                #[inline(always)]
                unsafe fn first_unchecked<ScalarLayout>(&self) -> $scalar_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>
                {
                    let shift = <$array_store as StaticStore>::CAPACITY_BITS.get() - ScalarLayout::BITS.get();
                    (*self >> shift) as $scalar_store
                }

                #[inline(always)]
                unsafe fn get_unchecked<ScalarLayout>(&self, pos: usize) -> $scalar_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>
                {
                    let shift = <$array_store as StaticStore>::CAPACITY_BITS.get() - ((pos + 1) * ScalarLayout::BITS.get());
                    ((*self >> shift) as $scalar_store) & ScalarLayout::MASK
                }
            }
        )+
    };
}

impl_scalar_get!(u8 => u8);
impl_scalar_get!(u16 => u8, u16);
impl_scalar_get!(u32 => u8, u16, u32);
impl_scalar_get!(u64 => u8, u16, u32, u64);
