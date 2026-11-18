use crate::store::{GrowRight, PushRight, StaticStore, WidenRight};
use crate::{len, scalar};

macro_rules! impl_push_right_same_store {
    ($array_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl PushRight<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn push_right_unchecked<ScalarLayout>(
                    &mut self,
                    len: usize,
                    scalar: $scalar_store,
                )
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let used = len * width;
                    let to_padding =
                        <$array_store as StaticStore>::CAPACITY_BITS.get() - (used + width);
                    let scalar = (scalar as $array_store) << to_padding;

                    *self |= scalar;
                }
            }

            unsafe impl GrowRight<$array_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn grow_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> $array_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    let mut store = self;
                    unsafe {
                        <Self as PushRight<$scalar_store>>::push_right_unchecked::<ScalarLayout>(
                            &mut store,
                            L::LEN.get(),
                            scalar,
                        )
                    };

                    store
                }
            }
        )+
    };
}

impl_push_right_same_store!(u8; scalar [u8]);
impl_push_right_same_store!(u16; scalar [u8, u16]);
impl_push_right_same_store!(u32; scalar [u8, u16, u32]);
impl_push_right_same_store!(u64; scalar [u8, u16, u32, u64]);

macro_rules! impl_push_right_widen_store {
    ($array_store:ty => $next_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl WidenRight<$next_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn widen_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> $next_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    let width = ScalarLayout::BITS.get();
                    let next_capacity = <$next_store as StaticStore>::CAPACITY_BITS.get();
                    let store_capacity = <$array_store as StaticStore>::CAPACITY_BITS.get();
                    let used = L::LEN.get() * width;
                    let to_padding = next_capacity - (used + width);

                    let scalar = (scalar as $next_store) << to_padding;
                    let shift = next_capacity - store_capacity;

                    ((self as $next_store) << shift) | scalar
                }
            }

            unsafe impl GrowRight<$next_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn grow_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> $next_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    unsafe {
                        <Self as WidenRight<$next_store, $scalar_store>>::widen_right_unchecked::<
                            ScalarLayout,
                            L,
                        >(self, scalar)
                    }
                }
            }
        )+
    };
}

impl_push_right_widen_store!(u8 => u16; scalar [u8]);
impl_push_right_widen_store!(u8 => u32; scalar [u8]);
impl_push_right_widen_store!(u8 => u64; scalar [u8]);
impl_push_right_widen_store!(u16 => u32; scalar [u8, u16]);
impl_push_right_widen_store!(u16 => u64; scalar [u8, u16]);
impl_push_right_widen_store!(u32 => u64; scalar [u8, u16, u32]);
