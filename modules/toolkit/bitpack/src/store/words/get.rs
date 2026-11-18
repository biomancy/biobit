use crate::scalar;
use crate::store::ScalarGet;

use super::store::WordStore;
use super::utils;

unsafe impl<W, ScalarStore> ScalarGet<ScalarStore> for W
where
    W: WordStore + ?Sized,
    ScalarStore: scalar::Store,
{
    #[inline(always)]
    unsafe fn first_unchecked<ScalarLayout>(&self) -> ScalarStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
    {
        let width = ScalarLayout::BITS.get();
        let words = self.as_words();
        let scalar = words[0] >> (u64::BITS as usize - width);
        unsafe { ScalarStore::from_u64_unchecked(scalar) }
    }

    #[inline(always)]
    unsafe fn get_unchecked<ScalarLayout>(&self, pos: usize) -> ScalarStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
    {
        let scalar = utils::get_unchecked(self.as_words(), ScalarLayout::BITS.get(), pos);
        ScalarStore::from_u64::<ScalarLayout>(scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;
    use std::fmt::Debug;

    fn sample(pos: usize, width: usize) -> u64 {
        let mask = testing::low_mask(width);
        let mixed = (pos as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .rotate_left((width % 63) as u32);

        mixed & mask
    }

    fn assert_get_matches_reference<const WIDTH: usize, ScalarStore, const N: usize>()
    where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug,
        u64: From<ScalarStore>,
        [u64; N]: ScalarGet<ScalarStore>,
    {
        let len = (N * u64::BITS as usize) / WIDTH;
        let scalars: Vec<_> = (0..len).map(|pos| sample(pos, WIDTH)).collect();
        let words = testing::words_from_scalars::<N>(WIDTH, &scalars);

        let first =
            unsafe { <[u64; N] as ScalarGet<ScalarStore>>::first_unchecked::<Bits<WIDTH>>(&words) };
        assert_eq!(u64::from(first), scalars[0]);

        for pos in 0..len {
            let actual = unsafe {
                <[u64; N] as ScalarGet<ScalarStore>>::get_unchecked::<Bits<WIDTH>>(&words, pos)
            };
            let expected = testing::get_reference(&words, WIDTH, pos);

            assert_eq!(
                u64::from(actual),
                expected,
                "width={WIDTH}, words={N}, pos={pos}"
            );
        }
    }

    #[test]
    fn get_matches_reference_for_representative_widths() {
        assert_get_matches_reference::<1, u8, 2>();
        assert_get_matches_reference::<2, u8, 2>();
        assert_get_matches_reference::<3, u8, 2>();
        assert_get_matches_reference::<4, u8, 2>();
        assert_get_matches_reference::<8, u8, 2>();
        assert_get_matches_reference::<9, u16, 2>();
        assert_get_matches_reference::<16, u16, 2>();
        assert_get_matches_reference::<31, u32, 2>();
        assert_get_matches_reference::<32, u32, 2>();
        assert_get_matches_reference::<63, u64, 2>();
        assert_get_matches_reference::<64, u64, 2>();
    }

    #[test]
    fn get_matches_reference_for_multiple_word_counts() {
        assert_get_matches_reference::<2, u8, 3>();
        assert_get_matches_reference::<9, u16, 3>();
        assert_get_matches_reference::<31, u32, 5>();
        assert_get_matches_reference::<64, u64, 5>();
    }

    #[test]
    fn get_reads_scalar_crossing_word_boundary() {
        let scalars = [0x001, 0x155, 0x0aa, 0x1ff, 0x000, 0x101, 0x077, 0x123];
        let words = testing::words_from_scalars::<2>(9, &scalars);
        let actual = unsafe { <[u64; 2] as ScalarGet<u16>>::get_unchecked::<Bits<9>>(&words, 7) };

        assert_eq!(actual, 0x123);
    }
}
