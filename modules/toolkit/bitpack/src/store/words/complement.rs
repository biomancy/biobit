use crate::scalar;
use crate::store::Complement;

use super::store::WordStoreMut;
use super::utils;

macro_rules! impl_complement {
    ($($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl<W> Complement<$scalar_store> for W
            where
                W: WordStoreMut + ?Sized,
            {
                #[inline(always)]
                unsafe fn complement_unchecked<ScalarLayout>(&mut self, len: usize)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>,
                {
                    let words = self.as_words_mut();
                    let used = len * ScalarLayout::BITS.get();
                    let active_words = used.div_ceil(u64::BITS as usize);
                    let tail = used % u64::BITS as usize;
                    let mut word = 0;

                    while word < active_words - 1 {
                        words[word] ^= u64::MAX;
                        word += 1;
                    }

                    if tail != 0 {
                        words[word] ^= utils::high_mask(tail);
                    } else {
                        words[word] ^= u64::MAX;
                    }
                }
            }
        )+
    };
}

impl_complement!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bits;
    use crate::store::words::testing;

    fn complement_reference(scalars: &mut [u64], width: usize) {
        let mask = testing::low_mask(width);

        for scalar in scalars {
            *scalar = !*scalar & mask;
        }
    }

    #[test]
    fn complement_flips_only_active_bits() {
        let mut scalars: Vec<_> = (0..33).map(|pos| (pos & 3) as u64).collect();
        let mut words = testing::words_from_scalars::<2>(2, &scalars);

        unsafe { words.complement_unchecked::<Bits<2>>(33) };
        complement_reference(&mut scalars, 2);

        assert_eq!(words, testing::words_from_scalars::<2>(2, &scalars));
    }

    #[test]
    fn complement_flips_full_words() {
        let mut scalars: Vec<_> = (0..64).map(|pos| (pos & 3) as u64).collect();
        let mut words = testing::words_from_scalars::<2>(2, &scalars);

        unsafe { words.complement_unchecked::<Bits<2>>(64) };
        complement_reference(&mut scalars, 2);

        assert_eq!(words, testing::words_from_scalars::<2>(2, &scalars));
    }

    #[test]
    fn complement_supports_wider_scalar_stores() {
        let mut scalars: Vec<_> = (0..10).map(|pos| ((pos * 97) & 0x01ff) as u64).collect();
        let mut words = testing::words_from_scalars::<2>(9, &scalars);

        unsafe { words.complement_unchecked::<Bits<9>>(10) };
        complement_reference(&mut scalars, 9);

        assert_eq!(words, testing::words_from_scalars::<2>(9, &scalars));
    }
}
