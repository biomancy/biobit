use crate::scalar;
use crate::store::{Pack, StaticStore};

#[inline(always)]
fn pack_tiled<ScalarStore, const N: usize>(scalars: &[ScalarStore], width: usize) -> [u64; N]
where
    ScalarStore: Copy,
    u64: From<ScalarStore>,
{
    let scalars_per_word = u64::BITS as usize / width;
    let mut words = [0; N];
    let mut pos = 0;
    let mut word = 0;

    while pos < scalars.len() {
        let mut shift = u64::BITS as usize;
        let end = usize::min(pos + scalars_per_word, scalars.len());

        while pos < end {
            shift -= width;
            words[word] |= u64::from(scalars[pos]) << shift;
            pos += 1;
        }
        word += 1;
    }
    words
}

#[inline(always)]
fn pack_carry<ScalarStore, const N: usize>(scalars: &[ScalarStore], width: usize) -> [u64; N]
where
    ScalarStore: Copy,
    u64: From<ScalarStore>,
{
    let mut words = [0; N];
    let mut word = 0;
    let mut current = 0;
    let mut remaining = u64::BITS as usize;

    for scalar in scalars.iter().copied() {
        let scalar = u64::from(scalar);

        if width <= remaining {
            remaining -= width;
            current |= scalar << remaining;

            if remaining == 0 {
                words[word] = current;
                word += 1;
                current = 0;
                remaining = u64::BITS as usize;
            }
        } else {
            let low_bits = width - remaining;

            current |= scalar >> low_bits;
            words[word] = current;
            word += 1;
            current = scalar << (u64::BITS as usize - low_bits);
            remaining = u64::BITS as usize - low_bits;
        }
    }

    if remaining != u64::BITS as usize {
        words[word] = current;
    }

    words
}

macro_rules! impl_pack_from {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<const N: usize> Pack<$scalar_store> for [u64; N]
            where
                [u64; N]: StaticStore,
                u64: From<$scalar_store>,
            {
                #[inline(always)]
                unsafe fn pack_unchecked<ScalarLayout>(
                    scalars: &[$scalar_store],
                ) -> Self
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let width = ScalarLayout::BITS.get();

                    if width.is_power_of_two() {
                        pack_tiled::<$scalar_store, N>(scalars.as_ref(), width)
                    } else {
                        pack_carry::<$scalar_store, N>(scalars.as_ref(), width)
                    }
                }
            }
        )+
    };
}

impl_pack_from!(u8, u16, u32, u64,);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::len;
    use crate::store::words::testing;
    use std::fmt::Debug;

    fn sample(pos: usize, width: usize) -> u64 {
        let mask = testing::low_mask(width);
        let mixed = (pos as u64)
            .wrapping_mul(0xa24b_aed4_963e_e407)
            .rotate_left((width % 57) as u32);

        mixed & mask
    }

    fn assert_pack_from_slice_matches_reference<const WIDTH: usize, ScalarStore, const N: usize>(
        len: usize,
    ) where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug + TryFrom<u64>,
        <ScalarStore as TryFrom<u64>>::Error: Debug,
        u64: From<ScalarStore>,
        [u64; N]: Pack<ScalarStore>,
    {
        let raw: Vec<_> = (0..len).map(|pos| sample(pos, WIDTH)).collect();
        let scalars: Vec<_> = raw
            .iter()
            .copied()
            .map(|scalar| ScalarStore::try_from(scalar).unwrap())
            .collect();
        let expected = testing::words_from_scalars::<N>(WIDTH, &raw);
        let actual =
            unsafe { <[u64; N] as Pack<ScalarStore>>::pack_unchecked::<Bits<WIDTH>>(&scalars) };

        assert_eq!(actual, expected, "width={WIDTH}, words={N}, len={len}");
    }

    fn assert_pack_from_array_matches_reference<
        const WIDTH: usize,
        ScalarStore,
        const N: usize,
        L,
    >()
    where
        Bits<WIDTH>: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store + Copy + Debug + TryFrom<u64>,
        <ScalarStore as TryFrom<u64>>::Error: Debug,
        u64: From<ScalarStore>,
        [u64; N]: Pack<ScalarStore>,
        L: len::Length,
    {
        let raw: Vec<_> = (0..L::LEN.get()).map(|pos| sample(pos, WIDTH)).collect();
        let scalars = L::array_from_fn(|pos| ScalarStore::try_from(raw[pos]).unwrap());
        let expected = testing::words_from_scalars::<N>(WIDTH, &raw);
        let actual = unsafe {
            <[u64; N] as Pack<ScalarStore>>::pack_unchecked::<Bits<WIDTH>>(scalars.as_ref())
        };

        assert_eq!(actual, expected, "width={WIDTH}, words={N}");
    }

    #[test]
    fn pack_from_slice_matches_reference_for_representative_widths() {
        assert_pack_from_slice_matches_reference::<1, u8, 2>(127);
        assert_pack_from_slice_matches_reference::<2, u8, 2>(63);
        assert_pack_from_slice_matches_reference::<3, u8, 2>(41);
        assert_pack_from_slice_matches_reference::<4, u8, 2>(31);
        assert_pack_from_slice_matches_reference::<8, u8, 2>(15);
        assert_pack_from_slice_matches_reference::<9, u16, 2>(13);
        assert_pack_from_slice_matches_reference::<16, u16, 2>(7);
        assert_pack_from_slice_matches_reference::<31, u32, 2>(3);
        assert_pack_from_slice_matches_reference::<32, u32, 2>(3);
        assert_pack_from_slice_matches_reference::<63, u64, 2>(1);
        assert_pack_from_slice_matches_reference::<64, u64, 2>(1);
    }

    #[test]
    fn pack_from_array_matches_reference_for_representative_widths() {
        assert_pack_from_array_matches_reference::<1, u8, 2, len::Len<128>>();
        assert_pack_from_array_matches_reference::<2, u8, 2, len::Len<64>>();
        assert_pack_from_array_matches_reference::<3, u8, 2, len::Len<42>>();
        assert_pack_from_array_matches_reference::<4, u8, 2, len::Len<32>>();
        assert_pack_from_array_matches_reference::<8, u8, 2, len::Len<16>>();
        assert_pack_from_array_matches_reference::<9, u16, 2, len::Len<14>>();
        assert_pack_from_array_matches_reference::<16, u16, 2, len::Len<8>>();
        assert_pack_from_array_matches_reference::<31, u32, 2, len::Len<4>>();
        assert_pack_from_array_matches_reference::<32, u32, 2, len::Len<4>>();
        assert_pack_from_array_matches_reference::<63, u64, 2, len::Len<2>>();
        assert_pack_from_array_matches_reference::<64, u64, 2, len::Len<2>>();
    }

    #[test]
    fn pack_supports_empty_slices() {
        let actual = unsafe { <[u64; 2] as Pack<u8>>::pack_unchecked::<Bits<2>>(&[]) };

        assert_eq!(actual, [0, 0]);
    }
}
