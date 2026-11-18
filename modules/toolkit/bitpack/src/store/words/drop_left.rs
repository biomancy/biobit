use crate::store::{DropLeft, NarrowLeft, ShrinkLeft, StaticStore};
use crate::{len, scalar};

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_drop_left_words {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> DropLeft<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
                #[inline(always)]
                unsafe fn drop_left_unchecked<ScalarLayout>(&mut self, _len: usize)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();

                    utils::shift_left(self.as_words_mut(), width);
                }
            }

            unsafe impl<const N: usize, const M: usize> NarrowLeft<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn narrow_left_unchecked<ScalarLayout, L>(self) -> [u64; M]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    debug_assert!(M + 1 == N);
                    let mut store = self;
                    unsafe {
                        <Self as DropLeft<$scalar_store>>::drop_left_unchecked::<ScalarLayout>(
                            &mut store,
                            L::LEN.get(),
                        )
                    };

                    utils::copy_prefix::<N, M>(&store)
                }
            }

            unsafe impl<const N: usize, const M: usize> ShrinkLeft<[u64; M], $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                [u64; M]: StaticStore,
            {
                #[inline(always)]
                unsafe fn shrink_left_unchecked<ScalarLayout, L>(self) -> [u64; M]
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    debug_assert!(M == N || M + 1 == N);
                    let mut store = self;
                    unsafe {
                        <Self as DropLeft<$scalar_store>>::drop_left_unchecked::<ScalarLayout>(
                            &mut store,
                            L::LEN.get(),
                        )
                    };

                    utils::copy_prefix::<N, M>(&store)
                }
            }
        )+
    };
}

impl_drop_left_words!(u8, u16, u32, u64);

macro_rules! impl_drop_left_words_to_u64 {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<const N: usize> NarrowLeft<u64, $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
            {
                #[inline(always)]
                unsafe fn narrow_left_unchecked<ScalarLayout, L>(self) -> u64
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    let width = ScalarLayout::BITS.get();

                    if width == u64::BITS as usize {
                        self[1]
                    } else {
                        (self[0] << width) | (self[1] >> (u64::BITS as usize - width))
                    }
                }
            }

            unsafe impl<const N: usize> ShrinkLeft<u64, $scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
            {
                #[inline(always)]
                unsafe fn shrink_left_unchecked<ScalarLayout, L>(self) -> u64
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                    L: len::Prev,
                {
                    unsafe {
                        <Self as NarrowLeft<u64, $scalar_store>>::narrow_left_unchecked::<
                            ScalarLayout,
                            L,
                        >(self)
                    }
                }
            }
        )+
    };
}

impl_drop_left_words_to_u64!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;

    #[test]
    fn drop_left_removes_first_when_shrinking_to_u64() {
        let scalars: Vec<_> = (0..33).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<2>(2, &scalars);
        let actual = unsafe {
            <[u64; 2] as NarrowLeft<u64, u8>>::narrow_left_unchecked::<Bits<2>, len::Len<33>>(
                source,
            )
        };

        assert_eq!(
            actual,
            testing::words_from_scalars::<1>(2, &scalars[1..])[0]
        );
    }

    #[test]
    fn drop_left_removes_first_64_bit_scalar_when_shrinking_to_u64() {
        let source = [0x1111_2222_3333_4444, 0x5555_6666_7777_8888];
        let actual = unsafe {
            <[u64; 2] as NarrowLeft<u64, u64>>::narrow_left_unchecked::<Bits<64>, len::Len<2>>(
                source,
            )
        };

        assert_eq!(actual, source[1]);
    }

    #[test]
    fn drop_left_removes_first_with_same_word_count() {
        let scalars: Vec<_> = (0..34).map(|pos| (pos & 3) as u64).collect();
        let mut actual = testing::words_from_scalars::<2>(2, &scalars);
        unsafe { actual.drop_left_unchecked::<Bits<2>>(34) };

        assert_eq!(actual, testing::words_from_scalars::<2>(2, &scalars[1..]));
    }

    #[test]
    fn drop_left_removes_first_when_shrinking_word_count() {
        let scalars: Vec<_> = (0..65).map(|pos| (pos & 3) as u64).collect();
        let source = testing::words_from_scalars::<3>(2, &scalars);
        let actual = unsafe {
            <[u64; 3] as NarrowLeft<[u64; 2], u8>>::narrow_left_unchecked::<Bits<2>, len::Len<65>>(
                source,
            )
        };

        assert_eq!(actual, testing::words_from_scalars::<2>(2, &scalars[1..]));
    }
}
