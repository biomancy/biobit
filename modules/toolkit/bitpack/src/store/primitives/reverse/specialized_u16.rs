use super::Reverse;
use crate::{Bits, scalar};

#[inline(always)]
const fn reverse_width_1(store: u16, len: usize) -> u16 {
    store.reverse_bits() << (u16::BITS as usize - len)
}

#[inline(always)]
const fn reverse_width_2(store: u16, len: usize) -> u16 {
    let store = ((store & 0xcccc) >> 2) | ((store & 0x3333) << 2);
    let store = ((store & 0xf0f0) >> 4) | ((store & 0x0f0f) << 4);
    let store = store.rotate_left(8);

    store << (u16::BITS as usize - (len * 2))
}

#[inline(always)]
const fn reverse_width_3(store: u16, len: usize) -> u16 {
    match len {
        2 => ((store & 0xe000) >> 3) | ((store & 0x1c00) << 3),
        3 => ((store & 0xe000) >> 6) | (store & 0x1c00) | ((store & 0x0380) << 6),
        4 => {
            ((store & 0xe000) >> 9)
                | ((store & 0x1c00) >> 3)
                | ((store & 0x0380) << 3)
                | ((store & 0x0070) << 9)
        }
        5 => {
            ((store & 0xe000) >> 12)
                | ((store & 0x1c00) >> 6)
                | (store & 0x0380)
                | ((store & 0x0070) << 6)
                | ((store & 0x000e) << 12)
        }
        _ => store,
    }
}

#[inline(always)]
const fn reverse_width_4(store: u16, len: usize) -> u16 {
    let store = ((store & 0xf0f0) >> 4) | ((store & 0x0f0f) << 4);
    let store = store.rotate_left(8);

    store << (u16::BITS as usize - (len * 4))
}

#[inline(always)]
const fn reverse_width_5(store: u16, len: usize) -> u16 {
    match len {
        2 => ((store & 0xf800) >> 5) | ((store & 0x07c0) << 5),
        3 => ((store & 0xf800) >> 10) | (store & 0x07c0) | ((store & 0x003e) << 10),
        _ => store,
    }
}

#[inline(always)]
const fn reverse_width_6(store: u16, _len: usize) -> u16 {
    ((store & 0xfc00) >> 6) | ((store & 0x03f0) << 6)
}

#[inline(always)]
const fn reverse_width_7(store: u16, _len: usize) -> u16 {
    ((store & 0xfe00) >> 7) | ((store & 0x01fc) << 7)
}

#[inline(always)]
const fn reverse_width_8(store: u16, _len: usize) -> u16 {
    store.rotate_left(8)
}

#[inline(always)]
const fn reverse_noop(store: u16, _len: usize) -> u16 {
    store
}

macro_rules! impl_reverse {
    (scalar $scalar_store:ty; $($bits:literal => $helper:ident),+ $(,)?) => {
        $(
            unsafe impl Reverse<Bits<$bits>, $scalar_store> for u16
            where
                Bits<$bits>: scalar::Layout<Store = $scalar_store>,
            {
                #[inline(always)]
                unsafe fn reverse_unchecked(&mut self, len: usize) {
                    unsafe {
                        std::hint::assert_unchecked(len > 1);
                        std::hint::assert_unchecked(len <= u16::BITS as usize / $bits);
                    }

                    *self = $helper(*self, len);
                }
            }
        )+
    };
}

impl_reverse!(
    scalar u8;
    1 => reverse_width_1,
    2 => reverse_width_2,
    3 => reverse_width_3,
    4 => reverse_width_4,
    5 => reverse_width_5,
    6 => reverse_width_6,
    7 => reverse_width_7,
    8 => reverse_width_8,
);

impl_reverse!(
    scalar u16;
    9 => reverse_noop,
    10 => reverse_noop,
    11 => reverse_noop,
    12 => reverse_noop,
    13 => reverse_noop,
    14 => reverse_noop,
    15 => reverse_noop,
    16 => reverse_noop,
);

#[cfg(test)]
mod tests {
    use super::{super::testing, *};

    const REVERSED_WIDTH_4: u16 = reverse_width_4(0x1234, 4);

    fn assert_exhaustive(width: usize, len: usize, helper: fn(u16, usize) -> u16) {
        let padding_mask = testing::low_mask(u16::BITS as usize - (width * len)) as u16;

        for store in 0..=u16::MAX {
            if store & padding_mask == 0 {
                let expected =
                    testing::reverse_reference(store as u64, u16::BITS as usize, width, len) as u16;

                assert_eq!(helper(store, len), expected);
            }
        }
    }

    fn assert_reverse_impl_samples<ScalarLayout, ScalarStore>()
    where
        u16: Reverse<ScalarLayout, ScalarStore>,
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store,
    {
        testing::assert_reverse_impl::<u16, ScalarLayout, ScalarStore>();

        let width = ScalarLayout::BITS.get();
        for len in 2..=(u16::BITS as usize / width) {
            testing::assert_samples(width, len, u16::BITS as usize, |store| {
                let mut store = store as u16;

                unsafe {
                    <u16 as Reverse<ScalarLayout, ScalarStore>>::reverse_unchecked(&mut store, len);
                }

                store as u64
            });
        }
    }

    #[test]
    fn helpers_are_const_usable() {
        assert_eq!(REVERSED_WIDTH_4, 0x4321);
    }

    #[test]
    fn helpers_match_reference_for_valid_lengths() {
        for len in 2..=16 {
            assert_exhaustive(1, len, reverse_width_1);
        }

        for len in 2..=8 {
            assert_exhaustive(2, len, reverse_width_2);
        }

        for len in 2..=5 {
            assert_exhaustive(3, len, reverse_width_3);
        }

        for len in 2..=4 {
            assert_exhaustive(4, len, reverse_width_4);
        }

        for len in 2..=3 {
            assert_exhaustive(5, len, reverse_width_5);
        }

        assert_exhaustive(6, 2, reverse_width_6);
        assert_exhaustive(7, 2, reverse_width_7);
        assert_exhaustive(8, 2, reverse_width_8);

        for width in 9..=16 {
            assert_exhaustive(width, 1, reverse_noop);
        }
    }

    #[test]
    fn reverse_impls_cover_all_u16_scalar_widths() {
        assert_reverse_impl_samples::<Bits<1>, u8>();
        assert_reverse_impl_samples::<Bits<2>, u8>();
        assert_reverse_impl_samples::<Bits<3>, u8>();
        assert_reverse_impl_samples::<Bits<4>, u8>();
        assert_reverse_impl_samples::<Bits<5>, u8>();
        assert_reverse_impl_samples::<Bits<6>, u8>();
        assert_reverse_impl_samples::<Bits<7>, u8>();
        assert_reverse_impl_samples::<Bits<8>, u8>();
        assert_reverse_impl_samples::<Bits<9>, u16>();
        assert_reverse_impl_samples::<Bits<10>, u16>();
        assert_reverse_impl_samples::<Bits<11>, u16>();
        assert_reverse_impl_samples::<Bits<12>, u16>();
        assert_reverse_impl_samples::<Bits<13>, u16>();
        assert_reverse_impl_samples::<Bits<14>, u16>();
        assert_reverse_impl_samples::<Bits<15>, u16>();
        assert_reverse_impl_samples::<Bits<16>, u16>();
    }
}
