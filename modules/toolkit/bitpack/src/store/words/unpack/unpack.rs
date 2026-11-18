use crate::scalar;
use crate::store::Unpack;

use super::super::store::WordStore;
use super::{carry, tiled};

#[inline(always)]
fn unpack_unchecked<ScalarLayout, ScalarStore>(words: &[u64], scalars: &mut [ScalarStore])
where
    ScalarLayout: scalar::Layout<Store = ScalarStore>,
    ScalarStore: scalar::Store,
{
    if ScalarLayout::BITS.is_power_of_two() {
        tiled::unpack_to_slice::<ScalarLayout, ScalarStore>(words, scalars);
    } else {
        carry::unpack_to_slice::<ScalarLayout, ScalarStore>(words, scalars);
    }
}

unsafe impl<W, ScalarStore> Unpack<ScalarStore> for W
where
    W: WordStore + ?Sized,
    ScalarStore: scalar::Store,
{
    #[inline(always)]
    unsafe fn unpack_unchecked<ScalarLayout>(&self, scalars: &mut [ScalarStore])
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
    {
        unpack_unchecked::<ScalarLayout, ScalarStore>(self.as_words(), scalars);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::words::testing;
    use crate::{Bits, Len, len};
    use std::fmt::Debug;

    fn sample(pos: usize, width: usize) -> u64 {
        let mask = testing::low_mask(width);
        let mixed = (pos as u64)
            .wrapping_mul(0xd6e8_feb8_6659_fd93)
            .rotate_left((width % 59) as u32);

        mixed & mask
    }

    fn assert_unpack_to_array_matches_reference<
        const WIDTH: usize,
        ScalarStore,
        const N: usize,
        L,
    >()
    where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug,
        u64: From<ScalarStore>,
        [u64; N]: Unpack<ScalarStore>,
        L: len::Length,
    {
        let len = L::LEN.get();
        let scalars: Vec<_> = (0..len).map(|pos| sample(pos, WIDTH)).collect();
        let words = testing::words_from_scalars::<N>(WIDTH, &scalars);
        let mut actual = L::array_from_fn(|_| <ScalarStore as scalar::Store>::ZERO);

        unsafe { words.unpack_unchecked::<Bits<WIDTH>>(actual.as_mut()) };

        for pos in 0..len {
            let expected = testing::get_reference(&words, WIDTH, pos);
            let actual = actual.as_ref()[pos];

            assert_eq!(
                u64::from(actual),
                expected,
                "width={WIDTH}, words={N}, len={len}, pos={pos}"
            );
        }
    }

    fn assert_unpack_to_slice_matches_reference<const WIDTH: usize, ScalarStore, const N: usize>(
        len: usize,
    ) where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug + Default,
        u64: From<ScalarStore>,
        [u64; N]: Unpack<ScalarStore>,
    {
        let scalars: Vec<_> = (0..len).map(|pos| sample(pos, WIDTH)).collect();
        let words = testing::words_from_scalars::<N>(WIDTH, &scalars);
        let mut actual = vec![ScalarStore::default(); len];

        unsafe { words.unpack_unchecked::<Bits<WIDTH>>(&mut actual) };

        for (pos, actual) in actual.into_iter().enumerate() {
            let expected = testing::get_reference(&words, WIDTH, pos);

            assert_eq!(
                u64::from(actual),
                expected,
                "width={WIDTH}, words={N}, len={len}, pos={pos}"
            );
        }
    }

    #[test]
    fn unpack_to_array_matches_reference_for_representative_widths() {
        assert_unpack_to_array_matches_reference::<1, u8, 2, Len<128>>();
        assert_unpack_to_array_matches_reference::<2, u8, 2, Len<64>>();
        assert_unpack_to_array_matches_reference::<3, u8, 2, Len<42>>();
        assert_unpack_to_array_matches_reference::<4, u8, 2, Len<32>>();
        assert_unpack_to_array_matches_reference::<8, u8, 2, Len<16>>();
        assert_unpack_to_array_matches_reference::<9, u16, 2, Len<14>>();
        assert_unpack_to_array_matches_reference::<16, u16, 2, Len<8>>();
        assert_unpack_to_array_matches_reference::<31, u32, 2, Len<4>>();
        assert_unpack_to_array_matches_reference::<32, u32, 2, Len<4>>();
        assert_unpack_to_array_matches_reference::<63, u64, 2, Len<2>>();
        assert_unpack_to_array_matches_reference::<64, u64, 2, Len<2>>();
    }

    #[test]
    fn unpack_to_array_matches_reference_for_multiple_word_counts() {
        assert_unpack_to_array_matches_reference::<2, u8, 3, Len<96>>();
        assert_unpack_to_array_matches_reference::<9, u16, 3, Len<21>>();
        assert_unpack_to_array_matches_reference::<31, u32, 5, Len<10>>();
        assert_unpack_to_array_matches_reference::<64, u64, 5, Len<5>>();
    }

    #[test]
    fn unpack_to_slice_matches_reference_for_representative_widths() {
        assert_unpack_to_slice_matches_reference::<1, u8, 2>(127);
        assert_unpack_to_slice_matches_reference::<2, u8, 2>(63);
        assert_unpack_to_slice_matches_reference::<3, u8, 2>(41);
        assert_unpack_to_slice_matches_reference::<4, u8, 2>(31);
        assert_unpack_to_slice_matches_reference::<8, u8, 2>(15);
        assert_unpack_to_slice_matches_reference::<9, u16, 2>(13);
        assert_unpack_to_slice_matches_reference::<16, u16, 2>(7);
        assert_unpack_to_slice_matches_reference::<31, u32, 2>(3);
        assert_unpack_to_slice_matches_reference::<32, u32, 2>(3);
        assert_unpack_to_slice_matches_reference::<63, u64, 2>(1);
        assert_unpack_to_slice_matches_reference::<64, u64, 2>(1);
    }

    #[test]
    fn unpack_to_slice_matches_reference_for_multiple_word_counts() {
        assert_unpack_to_slice_matches_reference::<2, u8, 3>(95);
        assert_unpack_to_slice_matches_reference::<9, u16, 3>(20);
        assert_unpack_to_slice_matches_reference::<31, u32, 5>(9);
        assert_unpack_to_slice_matches_reference::<64, u64, 5>(4);
    }

    #[test]
    fn unpack_reads_scalar_crossing_word_boundary() {
        let scalars = [0x001, 0x155, 0x0aa, 0x1ff, 0x000, 0x101, 0x077, 0x123];
        let words = testing::words_from_scalars::<2>(9, &scalars);
        let mut actual = [0u16; 8];

        unsafe { words.unpack_unchecked::<Bits<9>>(&mut actual) };

        assert_eq!(actual, scalars.map(|scalar| scalar as u16));
    }

    #[test]
    fn tiled_array_helper_is_const_capable() {
        const EMPTY: [u8; 0] = unsafe { tiled::unpack_to_array_u8::<Bits<2>, 0>(&[]) };
        assert!(EMPTY.is_empty());
        const U8: [u8; 32] = unsafe { tiled::unpack_to_array_u8::<Bits<2>, 32>(&[u64::MAX]) };
        const U16: [u16; 4] = unsafe { tiled::unpack_to_array_u16::<Bits<16>, 4>(&[u64::MAX]) };
        const U32: [u32; 2] = unsafe { tiled::unpack_to_array_u32::<Bits<32>, 2>(&[u64::MAX]) };
        const U64: [u64; 1] = unsafe { tiled::unpack_to_array_u64::<Bits<64>, 1>(&[u64::MAX]) };

        assert_eq!(U8, [3; 32]);
        assert_eq!(U16, [u16::MAX; 4]);
        assert_eq!(U32, [u32::MAX; 2]);
        assert_eq!(U64, [u64::MAX; 1]);
    }

    #[test]
    fn carry_array_helper_is_const_capable() {
        const U8: [u8; 22] =
            unsafe { carry::unpack_to_array_u8::<Bits<3>, 22>(&[u64::MAX, 3u64 << 62]) };
        const U16: [u16; 8] =
            unsafe { carry::unpack_to_array_u16::<Bits<9>, 8>(&[u64::MAX, u64::MAX]) };
        const U32: [u32; 3] =
            unsafe { carry::unpack_to_array_u32::<Bits<31>, 3>(&[u64::MAX, u64::MAX]) };
        const U64: [u64; 2] =
            unsafe { carry::unpack_to_array_u64::<Bits<63>, 2>(&[u64::MAX, u64::MAX]) };

        assert_eq!(U8, [7; 22]);
        assert_eq!(U16, [0x01ff; 8]);
        assert_eq!(U32, [0x7fff_ffff; 3]);
        assert_eq!(U64, [0x7fff_ffff_ffff_ffff; 2]);
    }
}
