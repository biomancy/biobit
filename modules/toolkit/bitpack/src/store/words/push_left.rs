use crate::store::{GrowLeft, PushLeft, StaticStore, WidenLeft};
use crate::{len, scalar};

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_push_left_words {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> PushLeft<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
                #[inline(always)]
                unsafe fn push_left_unchecked<ScalarLayout>(
                    &mut self,
                    len: usize,
                    scalar: $scalar_store,
                )
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();
                    let used = (len + 1) * width;
                    let words = self.as_words_mut();

                    utils::shift_right(words, width);
                    utils::clear_after(words, used);
                    utils::set_unchecked(words, width, 0, scalar as u64);
                }
            }

            unsafe impl<const N: usize, const M: usize> WidenLeft<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn widen_left_unchecked<ScalarLayout, L>(
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
                        <[u64; M] as PushLeft<$scalar_store>>::push_left_unchecked::<ScalarLayout>(
                            &mut next,
                            L::LEN.get(),
                            scalar,
                        )
                    };

                    next
                }
            }

            unsafe impl<const N: usize, const M: usize> GrowLeft<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn grow_left_unchecked<ScalarLayout, L>(
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
                        <[u64; M] as PushLeft<$scalar_store>>::push_left_unchecked::<ScalarLayout>(
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

impl_push_left_words!(u8, u16, u32, u64);

macro_rules! impl_push_left_primitive_to_words {
    ($source:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl WidenLeft<[u64; 2], $scalar_store> for $source {
                #[inline(always)]
                unsafe fn widen_left_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> [u64; 2]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    let width = ScalarLayout::BITS.get();
                    let used = (L::LEN.get() + 1) * width;
                    let mut next = utils::u64_to_words(self);

                    utils::shift_right(next.as_mut_slice(), width);
                    utils::clear_after(next.as_mut_slice(), used);
                    utils::set_unchecked(next.as_mut_slice(), width, 0, scalar as u64);

                    next
                }
            }

            unsafe impl GrowLeft<[u64; 2], $scalar_store> for $source {
                #[inline(always)]
                unsafe fn grow_left_unchecked<ScalarLayout, L>(
                    self,
                    scalar: $scalar_store,
                ) -> [u64; 2]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Next,
                {
                    unsafe {
                        <Self as WidenLeft<[u64; 2], $scalar_store>>::widen_left_unchecked::<
                            ScalarLayout,
                            L,
                        >(self, scalar)
                    }
                }
            }
        )+
    };
}

impl_push_left_primitive_to_words!(u64 => u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;

    #[test]
    fn push_left_prepends_when_growing_from_u64_to_words() {
        let scalars: Vec<_> = (0..32).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<1>(2, &scalars)[0];
        let actual = unsafe {
            <u64 as WidenLeft<[u64; 2], u8>>::widen_left_unchecked::<Bits<2>, len::Len<32>>(
                source, 3,
            )
        };

        let mut expected = vec![3];
        expected.extend_from_slice(&scalars);
        assert_eq!(actual, testing::words_from_scalars::<2>(2, &expected));
    }

    #[test]
    fn push_left_prepends_with_same_word_count() {
        let scalars: Vec<_> = (0..33).map(|pos| (pos & 3) as u64).collect();
        let mut actual = testing::words_from_scalars::<2>(2, &scalars);
        unsafe { actual.push_left_unchecked::<Bits<2>>(33, 3) };

        let mut expected = vec![3];
        expected.extend_from_slice(&scalars);
        assert_eq!(actual, testing::words_from_scalars::<2>(2, &expected));
    }

    #[test]
    fn push_left_prepends_when_growing_word_count() {
        let scalars: Vec<_> = (0..64).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<2>(2, &scalars);
        let actual = unsafe {
            <[u64; 2] as WidenLeft<[u64; 3], u8>>::widen_left_unchecked::<Bits<2>, len::Len<64>>(
                source, 3,
            )
        };

        let mut expected = vec![3];
        expected.extend_from_slice(&scalars);
        assert_eq!(actual, testing::words_from_scalars::<3>(2, &expected));
    }
}
