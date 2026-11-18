use crate::store::{DropRight, NarrowRight, ShrinkRight, StaticStore};
use crate::{len, scalar};

macro_rules! impl_drop_right_same_store {
    ($array_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl DropRight<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn drop_right_unchecked<ScalarLayout>(&mut self, len: usize)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let new_len = len - 1;

                    if new_len == 0 {
                        *self = <$array_store as StaticStore>::ZERO;
                        return;
                    }

                    let new_padding = <$array_store as StaticStore>::CAPACITY_BITS.get() - (new_len * width);

                    *self &= <$array_store>::MAX << new_padding;
                }
            }

            unsafe impl ShrinkRight<$array_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn shrink_right_unchecked<ScalarLayout, L>(self) -> $array_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    let mut store = self;
                    unsafe {
                        <Self as DropRight<$scalar_store>>::drop_right_unchecked::<ScalarLayout>(
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

impl_drop_right_same_store!(u8; scalar [u8]);
impl_drop_right_same_store!(u16; scalar [u8, u16]);
impl_drop_right_same_store!(u32; scalar [u8, u16, u32]);
impl_drop_right_same_store!(u64; scalar [u8, u16, u32, u64]);

macro_rules! impl_drop_right_shrink_store {
    ($array_store:ty => $prev_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl NarrowRight<$prev_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn narrow_right_unchecked<ScalarLayout, L>(self) -> $prev_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    let shift = <$array_store as StaticStore>::CAPACITY_BITS.get()
                        - <$prev_store as StaticStore>::CAPACITY_BITS.get();

                    (self >> shift) as $prev_store
                }
            }

            unsafe impl ShrinkRight<$prev_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn shrink_right_unchecked<ScalarLayout, L>(self) -> $prev_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    unsafe {
                        <Self as NarrowRight<$prev_store, $scalar_store>>::narrow_right_unchecked::<
                            ScalarLayout,
                            L,
                        >(self)
                    }
                }
            }
        )+
    };
}

impl_drop_right_shrink_store!(u16 => u8; scalar [u8]);
impl_drop_right_shrink_store!(u32 => u8; scalar [u8]);
impl_drop_right_shrink_store!(u32 => u16; scalar [u8, u16]);
impl_drop_right_shrink_store!(u64 => u8; scalar [u8]);
impl_drop_right_shrink_store!(u64 => u16; scalar [u8, u16]);
impl_drop_right_shrink_store!(u64 => u32; scalar [u8, u16, u32]);
