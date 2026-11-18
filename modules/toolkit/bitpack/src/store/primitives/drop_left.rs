use crate::store::{DropLeft, NarrowLeft, ShrinkLeft, StaticStore};
use crate::{len, scalar};

macro_rules! impl_drop_left_same_store {
    ($array_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl DropLeft<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn drop_left_unchecked<ScalarLayout>(&mut self, _len: usize)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    *self <<= ScalarLayout::BITS.get();
                }
            }

            unsafe impl ShrinkLeft<$array_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn shrink_left_unchecked<ScalarLayout, L>(self) -> $array_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    let mut store = self;
                    unsafe {
                        <Self as DropLeft<$scalar_store>>::drop_left_unchecked::<ScalarLayout>(
                            &mut store,
                            L::LEN.get(),
                        )
                    };

                    store
                }
            }
        )+
    };
}

impl_drop_left_same_store!(u8; scalar [u8]);
impl_drop_left_same_store!(u16; scalar [u8, u16]);
impl_drop_left_same_store!(u32; scalar [u8, u16, u32]);
impl_drop_left_same_store!(u64; scalar [u8, u16, u32, u64]);

macro_rules! impl_drop_left_shrink_store {
    ($array_store:ty => $prev_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl NarrowLeft<$prev_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn narrow_left_unchecked<ScalarLayout, L>(self) -> $prev_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    let width = ScalarLayout::BITS.get();
                    let store_capacity = <$array_store as StaticStore>::CAPACITY_BITS.get();
                    let prev_capacity = <$prev_store as StaticStore>::CAPACITY_BITS.get();
                    let shift = store_capacity - prev_capacity - width;

                    (self >> shift) as $prev_store
                }
            }

            unsafe impl ShrinkLeft<$prev_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn shrink_left_unchecked<ScalarLayout, L>(self) -> $prev_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    unsafe {
                        <Self as NarrowLeft<$prev_store, $scalar_store>>::narrow_left_unchecked::<
                            ScalarLayout,
                            L,
                        >(self)
                    }
                }
            }
        )+
    };
}

impl_drop_left_shrink_store!(u16 => u8; scalar [u8]);
impl_drop_left_shrink_store!(u32 => u8; scalar [u8]);
impl_drop_left_shrink_store!(u32 => u16; scalar [u8, u16]);
impl_drop_left_shrink_store!(u64 => u8; scalar [u8]);
impl_drop_left_shrink_store!(u64 => u16; scalar [u8, u16]);
impl_drop_left_shrink_store!(u64 => u32; scalar [u8, u16, u32]);
