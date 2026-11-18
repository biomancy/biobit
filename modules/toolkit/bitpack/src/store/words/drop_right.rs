use crate::store::{DropRight, NarrowRight, ShrinkRight, StaticStore};
use crate::{len, scalar};

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_drop_right_words {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> DropRight<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
                #[inline(always)]
                unsafe fn drop_right_unchecked<ScalarLayout>(&mut self, len: usize)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    utils::clear_after(self.as_words_mut(), len - 1, ScalarLayout::BITS.get());
                }
            }

            unsafe impl<const N: usize, const M: usize> NarrowRight<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn narrow_right_unchecked<ScalarLayout, L>(self) -> [u64; M]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    debug_assert!(M + 1 == N);
                    let mut prev = utils::copy_prefix::<N, M>(&self);
                    unsafe {
                        <[u64; M] as DropRight<$scalar_store>>::drop_right_unchecked::<ScalarLayout>(
                            &mut prev,
                            L::LEN.get(),
                        )
                    };

                    prev
                }
            }

            unsafe impl<const N: usize, const M: usize> ShrinkRight<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn shrink_right_unchecked<ScalarLayout, L>(self) -> [u64; M]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    debug_assert!(M == N || M + 1 == N);
                    let mut prev = utils::copy_prefix::<N, M>(&self);
                    unsafe {
                        <[u64; M] as DropRight<$scalar_store>>::drop_right_unchecked::<ScalarLayout>(
                            &mut prev,
                            L::LEN.get(),
                        )
                    };

                    prev
                }
            }
        )+
    };
}

impl_drop_right_words!(u8, u16, u32, u64);

macro_rules! impl_drop_right_words_to_u64 {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<const N: usize> NarrowRight<u64, $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
            {
                #[inline(always)]
                unsafe fn narrow_right_unchecked<ScalarLayout, L>(self) -> u64
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    let used = (L::LEN.get() - 1) * ScalarLayout::BITS.get();
                    self[0] & utils::high_mask(used)
                }
            }

            unsafe impl<const N: usize> ShrinkRight<u64, $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
            {
                #[inline(always)]
                unsafe fn shrink_right_unchecked<ScalarLayout, L>(self) -> u64
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    unsafe {
                        <Self as NarrowRight<u64, $scalar_store>>::narrow_right_unchecked::<
                            ScalarLayout,
                            L,
                        >(self)
                    }
                }
            }
        )+
    };
}

impl_drop_right_words_to_u64!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;

    #[test]
    fn drop_right_removes_last_when_shrinking_to_u64() {
        let scalars: Vec<_> = (0..33).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<2>(2, &scalars);
        let actual = unsafe {
            <[u64; 2] as NarrowRight<u64, u8>>::narrow_right_unchecked::<Bits<2>, len::Len<33>>(
                source,
            )
        };

        assert_eq!(
            actual,
            testing::words_from_scalars::<1>(2, &scalars[..32])[0]
        );
    }

    #[test]
    fn drop_right_removes_last_with_same_word_count() {
        let scalars: Vec<_> = (0..34).map(|pos| (pos & 3) as u64).collect();
        let mut actual = testing::words_from_scalars::<2>(2, &scalars);
        unsafe { actual.drop_right_unchecked::<Bits<2>>(34) };

        assert_eq!(actual, testing::words_from_scalars::<2>(2, &scalars[..33]));
    }

    #[test]
    fn drop_right_removes_last_when_shrinking_word_count() {
        let scalars: Vec<_> = (0..65).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<3>(2, &scalars);
        let actual = unsafe {
            <[u64; 3] as NarrowRight<[u64; 2], u8>>::narrow_right_unchecked::<Bits<2>, len::Len<65>>(
                source,
            )
        };

        assert_eq!(actual, testing::words_from_scalars::<2>(2, &scalars[..64]));
    }
}
