use crate::scalar;
use crate::store::ScalarSet;

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_scalar_set {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> ScalarSet<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
                #[inline(always)]
                unsafe fn set_unchecked<ScalarLayout>(
                    &mut self,
                    pos: usize,
                    scalar: $scalar_store,
                ) where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    utils::set_unchecked(
                        self.as_words_mut(),
                        ScalarLayout::BITS.get(),
                        pos,
                        scalar as u64,
                    );
                }
            }
        )+
    };
}

impl_scalar_set!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::words::testing;
    use crate::{Bits, store::ScalarGet};
    use std::fmt::Debug;

    fn sample(pos: usize, width: usize) -> u64 {
        let mask = testing::low_mask(width);
        let mixed = (!pos as u64)
            .wrapping_mul(0xbf58_476d_1ce4_e5b9)
            .rotate_left((width % 61) as u32);

        mixed & mask
    }

    fn assert_set_matches_reference<const WIDTH: usize, ScalarStore, const N: usize>()
    where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug,
        u64: From<ScalarStore>,
        ScalarStore: TryFrom<u64>,
        <ScalarStore as TryFrom<u64>>::Error: Debug,
        [u64; N]: ScalarGet<ScalarStore> + ScalarSet<ScalarStore>,
    {
        let len = (N * u64::BITS as usize) / WIDTH;
        let initial: Vec<_> = (0..len).map(|pos| sample(pos, WIDTH)).collect();
        let mut words = testing::words_from_scalars::<N>(WIDTH, &initial);

        for pos in 0..len {
            let scalar = sample(pos + len, WIDTH);
            let typed = ScalarStore::try_from(scalar).unwrap();
            let mut expected = words;

            testing::set_reference(&mut expected, WIDTH, pos, scalar);
            unsafe { words.set_unchecked::<Bits<WIDTH>>(pos, typed) };

            assert_eq!(words, expected, "width={WIDTH}, words={N}, pos={pos}");

            let actual = unsafe {
                <[u64; N] as ScalarGet<ScalarStore>>::get_unchecked::<Bits<WIDTH>>(&words, pos)
            };
            assert_eq!(u64::from(actual), scalar);
        }
    }

    #[test]
    fn set_matches_reference_for_representative_widths() {
        assert_set_matches_reference::<1, u8, 2>();
        assert_set_matches_reference::<2, u8, 2>();
        assert_set_matches_reference::<3, u8, 2>();
        assert_set_matches_reference::<4, u8, 2>();
        assert_set_matches_reference::<8, u8, 2>();
        assert_set_matches_reference::<9, u16, 2>();
        assert_set_matches_reference::<16, u16, 2>();
        assert_set_matches_reference::<31, u32, 2>();
        assert_set_matches_reference::<32, u32, 2>();
        assert_set_matches_reference::<63, u64, 2>();
        assert_set_matches_reference::<64, u64, 2>();
    }

    #[test]
    fn set_matches_reference_for_multiple_word_counts() {
        assert_set_matches_reference::<2, u8, 3>();
        assert_set_matches_reference::<9, u16, 3>();
        assert_set_matches_reference::<31, u32, 5>();
        assert_set_matches_reference::<64, u64, 5>();
    }

    #[test]
    fn set_replaces_scalar_crossing_word_boundary() {
        let scalars = [0x001, 0x155, 0x0aa, 0x1ff, 0x000, 0x101, 0x077, 0x123];
        let mut words = testing::words_from_scalars::<2>(9, &scalars);
        let mut expected = words;

        testing::set_reference(&mut expected, 9, 7, 0x1ab);
        unsafe { words.set_unchecked::<Bits<9>>(7, 0x1abu16) };

        assert_eq!(words, expected);
        assert_eq!(
            unsafe { <[u64; 2] as ScalarGet<u16>>::get_unchecked::<Bits<9>>(&words, 7) },
            0x1ab
        );
    }
}
