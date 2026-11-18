use super::Reverse;
use crate::{Bits, scalar};

#[inline(always)]
const fn reverse_width_1(store: u8, len: usize) -> u8 {
    store.reverse_bits() << (u8::BITS as usize - len)
}

#[inline(always)]
const fn reverse_width_2(store: u8, len: usize) -> u8 {
    let store = ((store & 0b1100_1100) >> 2) | ((store & 0b0011_0011) << 2);
    let store = ((store & 0b1111_0000) >> 4) | ((store & 0b0000_1111) << 4);

    store << (u8::BITS as usize - (len * 2))
}

#[inline(always)]
const fn reverse_width_3(store: u8, _len: usize) -> u8 {
    ((store & 0b1110_0000) >> 3) | ((store & 0b0001_1100) << 3)
}

#[inline(always)]
const fn reverse_width_4(store: u8, _len: usize) -> u8 {
    store.rotate_left(4)
}

#[inline(always)]
const fn reverse_noop(store: u8, _len: usize) -> u8 {
    store
}

macro_rules! impl_reverse {
    ($($bits:literal => $helper:ident),+ $(,)?) => {
        $(
            unsafe impl Reverse<Bits<$bits>, u8> for u8
            where
                Bits<$bits>: scalar::Layout<Store = u8>,
            {
                #[inline(always)]
                unsafe fn reverse_unchecked(&mut self, len: usize) {
                    unsafe {
                        std::hint::assert_unchecked(len > 1);
                        std::hint::assert_unchecked(len <= u8::BITS as usize / $bits);
                    }

                    *self = $helper(*self, len);
                }
            }
        )+
    };
}

impl_reverse!(
    1 => reverse_width_1,
    2 => reverse_width_2,
    3 => reverse_width_3,
    4 => reverse_width_4,
    5 => reverse_noop,
    6 => reverse_noop,
    7 => reverse_noop,
    8 => reverse_noop,
);

#[cfg(test)]
mod tests {
    use super::{super::testing, *};

    const REVERSED_WIDTH_2: u8 = reverse_width_2(0b00_01_10_11, 4);

    fn assert_exhaustive(width: usize, len: usize, helper: fn(u8, usize) -> u8) {
        let padding_mask = testing::low_mask(u8::BITS as usize - (width * len)) as u8;

        for store in 0..=u8::MAX {
            if store & padding_mask == 0 {
                let expected =
                    testing::reverse_reference(store as u64, u8::BITS as usize, width, len) as u8;

                assert_eq!(helper(store, len), expected);
            }
        }
    }

    fn assert_reverse_impl_samples<ScalarLayout, ScalarStore>()
    where
        u8: Reverse<ScalarLayout, ScalarStore>,
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store,
    {
        testing::assert_reverse_impl::<u8, ScalarLayout, ScalarStore>();

        let width = ScalarLayout::BITS.get();
        for len in 2..=(u8::BITS as usize / width) {
            testing::assert_samples(width, len, u8::BITS as usize, |store| {
                let mut store = store as u8;

                unsafe {
                    <u8 as Reverse<ScalarLayout, ScalarStore>>::reverse_unchecked(&mut store, len);
                }

                store as u64
            });
        }
    }

    #[test]
    fn helpers_are_const_usable() {
        assert_eq!(REVERSED_WIDTH_2, 0b11_10_01_00);
    }

    #[test]
    fn helpers_match_reference_for_valid_lengths() {
        for len in 2..=8 {
            assert_exhaustive(1, len, reverse_width_1);
        }

        for len in 2..=4 {
            assert_exhaustive(2, len, reverse_width_2);
        }

        assert_exhaustive(3, 2, reverse_width_3);
        assert_exhaustive(4, 2, reverse_width_4);

        for width in 5..=8 {
            assert_exhaustive(width, 1, reverse_noop);
        }
    }

    #[test]
    fn reverse_impls_cover_all_u8_scalar_widths() {
        assert_reverse_impl_samples::<Bits<1>, u8>();
        assert_reverse_impl_samples::<Bits<2>, u8>();
        assert_reverse_impl_samples::<Bits<3>, u8>();
        assert_reverse_impl_samples::<Bits<4>, u8>();
        assert_reverse_impl_samples::<Bits<5>, u8>();
        assert_reverse_impl_samples::<Bits<6>, u8>();
        assert_reverse_impl_samples::<Bits<7>, u8>();
        assert_reverse_impl_samples::<Bits<8>, u8>();
    }
}
