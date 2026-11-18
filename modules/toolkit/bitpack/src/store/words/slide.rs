use crate::store::Slide;
use crate::{len, scalar};

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_slide {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> Slide<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
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
                    let used = len * width;
                    let words = self.as_words_mut();

                    utils::shift_right(words, width);
                    utils::clear_after(words, used);
                    utils::set_unchecked(words, width, 0, scalar as u64);
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
                    let used = len * width;
                    let words = self.as_words_mut();

                    utils::shift_left(words, width);
                    utils::clear_after(words, used);
                    utils::set_unchecked(words, width, len - 1, scalar as u64);
                }
            }
        )+
    };
}

impl_slide!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;
    use std::fmt::Debug;

    fn sample(pos: usize, width: usize) -> u64 {
        let mask = testing::low_mask(width);
        let mixed = (pos as u64)
            .wrapping_mul(0x94d0_49bb_1331_11eb)
            .rotate_left((width % 53) as u32);

        mixed & mask
    }

    fn assert_slide_matches_reference<const WIDTH: usize, ScalarStore, const N: usize, L>()
    where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug + TryFrom<u64>,
        <ScalarStore as TryFrom<u64>>::Error: Debug,
        [u64; N]: Slide<ScalarStore>,
        L: len::Length,
    {
        let len = L::LEN.get();
        let raw: Vec<_> = (0..len).map(|pos| sample(pos, WIDTH)).collect();
        let scalar = sample(len, WIDTH);
        let typed = ScalarStore::try_from(scalar).unwrap();

        let mut actual = testing::words_from_scalars::<N>(WIDTH, &raw);
        unsafe { actual.slide_left_unchecked::<Bits<WIDTH>, L>(typed) };

        let mut expected = Vec::with_capacity(len);
        expected.push(scalar);
        expected.extend_from_slice(&raw[..len - 1]);
        assert_eq!(
            actual,
            testing::words_from_scalars::<N>(WIDTH, &expected),
            "slide_left width={WIDTH}, words={N}, len={len}"
        );

        let mut actual = testing::words_from_scalars::<N>(WIDTH, &raw);
        unsafe { actual.slide_right_unchecked::<Bits<WIDTH>, L>(typed) };

        let mut expected = Vec::with_capacity(len);
        expected.extend_from_slice(&raw[1..]);
        expected.push(scalar);
        assert_eq!(
            actual,
            testing::words_from_scalars::<N>(WIDTH, &expected),
            "slide_right width={WIDTH}, words={N}, len={len}"
        );
    }

    #[test]
    fn slide_matches_reference_for_representative_widths() {
        assert_slide_matches_reference::<1, u8, 2, len::Len<128>>();
        assert_slide_matches_reference::<2, u8, 2, len::Len<64>>();
        assert_slide_matches_reference::<3, u8, 2, len::Len<42>>();
        assert_slide_matches_reference::<9, u16, 2, len::Len<14>>();
        assert_slide_matches_reference::<31, u32, 2, len::Len<4>>();
        assert_slide_matches_reference::<64, u64, 2, len::Len<2>>();
    }

    #[test]
    fn slide_matches_reference_for_partial_word_lengths() {
        assert_slide_matches_reference::<2, u8, 3, len::Len<65>>();
        assert_slide_matches_reference::<9, u16, 3, len::Len<20>>();
        assert_slide_matches_reference::<31, u32, 5, len::Len<9>>();
        assert_slide_matches_reference::<63, u64, 5, len::Len<4>>();
    }
}
