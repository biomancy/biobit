use crate::store::{Slide, StaticStore};
use crate::{len, scalar};

macro_rules! impl_slide {
    ($array_store:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl Slide<$scalar_store> for $array_store {
                #[inline(always)]
                unsafe fn slide_left_unchecked<ScalarLayout, L>(
                    &mut self,
                    scalar: $scalar_store,
                ) where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Length,
                {
                    unsafe {
                        self.slide_left_runtime_unchecked::<ScalarLayout>(
                            L::LEN.get(),
                            scalar,
                        )
                    };
                }

                #[inline(always)]
                unsafe fn slide_left_runtime_unchecked<ScalarLayout>(
                    &mut self,
                    len: usize,
                    scalar: $scalar_store,
                ) where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let capacity = <$array_store as StaticStore>::CAPACITY_BITS.get();
                    let padding = capacity - (len * width);
                    let scalar = (scalar as $array_store) << (capacity - width);

                    if width == capacity {
                        *self = scalar;
                    } else if padding == 0 {
                        *self = (*self >> width) | scalar;
                    } else {
                        *self = ((*self >> width) & (<$array_store>::MAX << padding)) | scalar;
                    }
                }

                #[inline(always)]
                unsafe fn slide_right_unchecked<ScalarLayout, L>(
                    &mut self,
                    scalar: $scalar_store,
                ) where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Length,
                {
                    unsafe {
                        self.slide_right_runtime_unchecked::<ScalarLayout>(
                            L::LEN.get(),
                            scalar,
                        )
                    };
                }

                #[inline(always)]
                unsafe fn slide_right_runtime_unchecked<ScalarLayout>(
                    &mut self,
                    len: usize,
                    scalar: $scalar_store,
                ) where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let padding = <$array_store as StaticStore>::CAPACITY_BITS.get() - (len * width);

                    let scalar = (scalar as $array_store) << padding;

                    if len == 1 {
                        *self = scalar;
                    } else {
                        *self <<= width;
                        *self |= scalar;
                    }
                }
            }
        )+
    };
}

impl_slide!(u8 => u8);
impl_slide!(u16 => u8, u16);
impl_slide!(u32 => u8, u16, u32);
impl_slide!(u64 => u8, u16, u32, u64);
