use crate::store::{GrowLeft, PushLeft, StaticStore, WidenLeft};
use crate::{len, scalar};

macro_rules! impl_push_left_same_store {
    ($array_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl PushLeft<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn push_left_unchecked<ScalarLayout>(
                    &mut self,
                    _len: usize,
                    scalar: $scalar_store,
                )
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let capacity = <$array_store as StaticStore>::CAPACITY_BITS.get();

                    if width == capacity {
                        *self = scalar as $array_store;
                    } else {
                        *self = ((scalar as $array_store) << (capacity - width)) | (*self >> width);
                    }
                }
            }

            unsafe impl GrowLeft<$array_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn grow_left_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> $array_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    let mut store = self;
                    unsafe {
                        <Self as PushLeft<$scalar_store>>::push_left_unchecked::<ScalarLayout>(
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

impl_push_left_same_store!(u8; scalar [u8]);
impl_push_left_same_store!(u16; scalar [u8, u16]);
impl_push_left_same_store!(u32; scalar [u8, u16, u32]);
impl_push_left_same_store!(u64; scalar [u8, u16, u32, u64]);

macro_rules! impl_push_left_widen_store {
    ($array_store:ty => $next_store:ty; scalar [$($scalar_store:ty),+ $(,)?]) => {
        $(
            unsafe impl WidenLeft<$next_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn widen_left_unchecked<ScalarLayout, L>(
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

                    let scalar = (scalar as $next_store) << (next_capacity - width);
                    let shift = next_capacity - store_capacity - width;

                    ((self as $next_store) << shift) | scalar
                }
            }

            unsafe impl GrowLeft<$next_store, $scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn grow_left_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> $next_store
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    unsafe {
                        <Self as WidenLeft<$next_store, $scalar_store>>::widen_left_unchecked::<
                            ScalarLayout,
                            L,
                        >(self, scalar)
                    }
                }
            }
        )+
    };
}

impl_push_left_widen_store!(u8 => u16; scalar [u8]);
impl_push_left_widen_store!(u8 => u32; scalar [u8]);
impl_push_left_widen_store!(u8 => u64; scalar [u8]);
impl_push_left_widen_store!(u16 => u32; scalar [u8, u16]);
impl_push_left_widen_store!(u16 => u64; scalar [u8, u16]);
impl_push_left_widen_store!(u32 => u64; scalar [u8, u16, u32]);
