use crate::store::{GrowRight, PushRight, StaticStore, WidenRight};
use crate::{len, scalar};

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_push_right_words {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> PushRight<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
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
                    let words = self.as_words_mut();

                    utils::set_unchecked(words, width, len, scalar as u64);
                    utils::clear_after(words, len + 1, width);
                }
            }

            unsafe impl<const N: usize, const M: usize> WidenRight<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn widen_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> [u64; M]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    debug_assert!(M == N + 1);
                    let mut next = utils::copy_prefix::<N, M>(&self);
                    unsafe {
                        <[u64; M] as PushRight<$scalar_store>>::push_right_unchecked::<ScalarLayout>(
                            &mut next,
                            L::LEN.get(),
                            scalar,
                        )
                    };

                    next
                }
            }

            unsafe impl<const N: usize, const M: usize> GrowRight<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn grow_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> [u64; M]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    debug_assert!(M == N || M == N + 1);
                    let mut next = utils::copy_prefix::<N, M>(&self);
                    unsafe {
                        <[u64; M] as PushRight<$scalar_store>>::push_right_unchecked::<ScalarLayout>(
                            &mut next,
                            L::LEN.get(),
                            scalar,
                        )
                    };

                    next
                }
            }
        )+
    };
}

impl_push_right_words!(u8, u16, u32, u64);

macro_rules! impl_push_right_primitive_to_words {
    ($source:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl WidenRight<[u64; 2], $scalar_store> for $source {
                #[inline(always)]
                unsafe fn widen_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> [u64; 2]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    let width = ScalarLayout::BITS.get();
                    let mut next = [self, 0];

                    utils::set_unchecked(next.as_mut_slice(), width, L::LEN.get(), scalar as u64);
                    utils::clear_after(next.as_mut_slice(), L::LEN.get() + 1, width);

                    next
                }
            }

            unsafe impl GrowRight<[u64; 2], $scalar_store> for $source {
                #[inline(always)]
                unsafe fn grow_right_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> [u64; 2]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    unsafe {
                        <Self as WidenRight<[u64; 2], $scalar_store>>::widen_right_unchecked::<
                            ScalarLayout,
                            L,
                        >(self, scalar)
                    }
                }
            }
        )+
    };
}

impl_push_right_primitive_to_words!(u64 => u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;

    #[test]
    fn push_right_appends_when_growing_from_u64_to_words() {
        let scalars: Vec<_> = (0..32).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<1>(2, &scalars)[0];
        let actual = unsafe {
            <u64 as WidenRight<[u64; 2], u8>>::widen_right_unchecked::<Bits<2>, len::Len<32>>(
                source, 3,
            )
        };

        let mut expected = scalars;
        expected.push(3);
        assert_eq!(actual, testing::words_from_scalars::<2>(2, &expected));
    }

    #[test]
    fn push_right_appends_with_same_word_count() {
        let scalars: Vec<_> = (0..33).map(|pos| (pos & 3) as u64).collect();
        let mut actual = testing::words_from_scalars::<2>(2, &scalars);
        unsafe { actual.push_right_unchecked::<Bits<2>>(33, 3) };

        let mut expected = scalars;
        expected.push(3);
        assert_eq!(actual, testing::words_from_scalars::<2>(2, &expected));
    }

    #[test]
    fn push_right_appends_when_growing_word_count() {
        let scalars: Vec<_> = (0..64).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<2>(2, &scalars);
        let actual = unsafe {
            <[u64; 2] as WidenRight<[u64; 3], u8>>::widen_right_unchecked::<Bits<2>, len::Len<64>>(
                source, 3,
            )
        };

        let mut expected = scalars;
        expected.push(3);
        assert_eq!(actual, testing::words_from_scalars::<3>(2, &expected));
    }
}
