use crate::store::{StaticStore, Store};
use std::num::NonZeroUsize;

macro_rules! impl_store {
    ($($store:ty),+ $(,)?) => {
        $(
            impl Store for $store {
                #[inline(always)]
                fn capacity_bits(&self) -> usize {
                    <$store as StaticStore>::CAPACITY_BITS.get()
                }
            }

            impl StaticStore for $store {
                const ZERO: Self = 0;
                const CAPACITY_BITS: NonZeroUsize =
                    NonZeroUsize::new(<$store>::BITS as usize).unwrap();
            }
        )+
    };
}

impl_store!(u8, u16, u32, u64);
