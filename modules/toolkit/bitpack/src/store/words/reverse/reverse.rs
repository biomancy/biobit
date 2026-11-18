use crate::store::Reverse;
use crate::{Bits, scalar};

use super::super::store::WordStoreMut;
use super::super::utils::WordOffset;
use super::{generic, tiled};

#[inline(always)]
fn reverse_unchecked(words: &mut [u64], width: usize, len: usize) {
    let end = WordOffset::new(len, width);
    debug_assert!(len > 1 && end.index + usize::from(end.offset != 0) <= words.len());

    if width.is_power_of_two() && end.offset == 0 {
        tiled::reverse_full_words(words, end.index, width);
    } else {
        generic::reverse_fields(words, width, len);
    }
}

macro_rules! impl_reverse {
    (scalar $scalar_store:ty; $($bits:literal),+ $(,)?) => {
        $(
            unsafe impl<W> Reverse<Bits<$bits>, $scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
                Bits<$bits>: scalar::Layout<Store = $scalar_store>,
            {
                #[inline(always)]
                unsafe fn reverse_unchecked(&mut self, len: usize) {
                    let words = self.as_words_mut();

                    reverse_unchecked(words, $bits, len);
                }
            }
        )+
    };
}

impl_reverse!(scalar u8; 1, 2, 3, 4, 5, 6, 7, 8);
impl_reverse!(scalar u16; 9, 10, 11, 12, 13, 14, 15, 16);
impl_reverse!(scalar u32; 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);
impl_reverse!(
    scalar u64;
    33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48,
    49, 50, 51, 52, 53, 54, 55, 56,
    57, 58, 59, 60, 61, 62, 63, 64,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::words::testing;

    fn sample(pos: usize, width: usize) -> u64 {
        let mask = testing::low_mask(width);
        let mixed = (pos as u64)
            .wrapping_mul(0xbf58_476d_1ce4_e5b9)
            .rotate_left((width % 47) as u32);

        mixed & mask
    }

    fn assert_reverse_matches_reference<ScalarLayout, ScalarStore, const N: usize>(
        width: usize,
        len: usize,
    ) where
        [u64; N]: Reverse<ScalarLayout, ScalarStore>,
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store,
    {
        let scalars: Vec<_> = (0..len).map(|pos| sample(pos, width)).collect();
        let mut words = testing::words_from_scalars::<N>(width, &scalars);
        let expected = testing::reverse_reference(&words, width, len);

        unsafe {
            <[u64; N] as Reverse<ScalarLayout, ScalarStore>>::reverse_unchecked(&mut words, len)
        };

        assert_eq!(words, expected, "width={width}, words={N}, len={len}");
    }

    #[test]
    fn reverse_matches_reference_for_full_word_tiled_cases() {
        assert_reverse_matches_reference::<Bits<1>, u8, 2>(1, 128);
        assert_reverse_matches_reference::<Bits<2>, u8, 2>(2, 64);
        assert_reverse_matches_reference::<Bits<4>, u8, 2>(4, 32);
        assert_reverse_matches_reference::<Bits<8>, u8, 2>(8, 16);
        assert_reverse_matches_reference::<Bits<16>, u16, 2>(16, 8);
        assert_reverse_matches_reference::<Bits<32>, u32, 2>(32, 4);
        assert_reverse_matches_reference::<Bits<64>, u64, 2>(64, 2);
    }

    #[test]
    fn reverse_matches_reference_for_partial_and_carry_cases() {
        assert_reverse_matches_reference::<Bits<2>, u8, 2>(2, 33);
        assert_reverse_matches_reference::<Bits<3>, u8, 2>(3, 42);
        assert_reverse_matches_reference::<Bits<9>, u16, 3>(9, 20);
        assert_reverse_matches_reference::<Bits<31>, u32, 5>(31, 9);
        assert_reverse_matches_reference::<Bits<63>, u64, 5>(63, 4);
    }
}
