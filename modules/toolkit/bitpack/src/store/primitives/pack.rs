use crate::scalar;
use crate::store::{Pack, StaticStore};

macro_rules! impl_pack_from {
    ($array_store:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl Pack<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn pack_unchecked<ScalarLayout>(
                    scalars: &[$scalar_store],
                ) -> Self
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let mut iter = scalars.iter();
                    let Some(first) = iter.next() else {
                        return <$array_store as StaticStore>::ZERO;
                    };
                    let mut store = *first as $array_store;

                    let width = ScalarLayout::BITS.get();
                    for scalar in iter {
                        store = (store << width) | (*scalar as $array_store);
                    }

                    store <<= <$array_store as StaticStore>::CAPACITY_BITS.get() - (scalars.len() * width);
                    store
                }
            }
        )+
    };
}

impl_pack_from!(u8 => u8);
impl_pack_from!(u16 => u8, u16);
impl_pack_from!(u32 => u8, u16, u32);
impl_pack_from!(u64 => u8, u16, u32, u64);
