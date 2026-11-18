use super::Reverse;
use crate::{Bits, scalar};

#[inline(always)]
const fn scalar_mask<const WIDTH: usize>() -> u32 {
    if WIDTH == u32::BITS as usize {
        u32::MAX
    } else {
        (1u32 << WIDTH) - 1
    }
}

#[inline(always)]
const fn reverse_fields<const WIDTH: usize>(store: u32, len: usize) -> u32 {
    let mask = scalar_mask::<WIDTH>();
    let mut reversed = 0;
    let mut pos = 0;

    while pos < len {
        let src_shift = u32::BITS as usize - ((pos + 1) * WIDTH);
        let scalar = (store >> src_shift) & mask;
        let dst_shift = u32::BITS as usize - ((len - pos) * WIDTH);

        reversed |= scalar << dst_shift;
        pos += 1;
    }

    reversed
}

#[inline(always)]
const fn reverse_two_fields<const WIDTH: usize>(store: u32) -> u32 {
    let mask = scalar_mask::<WIDTH>();
    let high = mask << (u32::BITS as usize - WIDTH);
    let low = mask << (u32::BITS as usize - (WIDTH * 2));

    ((store & high) >> WIDTH) | ((store & low) << WIDTH)
}

#[inline(always)]
const fn reverse_three_fields<const WIDTH: usize>(store: u32) -> u32 {
    let mask = scalar_mask::<WIDTH>();
    let high = mask << (u32::BITS as usize - WIDTH);
    let middle = mask << (u32::BITS as usize - (WIDTH * 2));
    let low = mask << (u32::BITS as usize - (WIDTH * 3));

    ((store & high) >> (WIDTH * 2)) | (store & middle) | ((store & low) << (WIDTH * 2))
}

#[inline(always)]
const fn reverse_two_or_three_fields<const WIDTH: usize>(store: u32, len: usize) -> u32 {
    match len {
        2 => reverse_two_fields::<WIDTH>(store),
        3 => reverse_three_fields::<WIDTH>(store),
        _ => store,
    }
}

#[inline(always)]
const fn reverse_width_1(store: u32, len: usize) -> u32 {
    store.reverse_bits() << (u32::BITS as usize - len)
}

#[inline(always)]
const fn reverse_width_2(store: u32, len: usize) -> u32 {
    let store = ((store & 0xcccc_cccc) >> 2) | ((store & 0x3333_3333) << 2);
    let store = ((store & 0xf0f0_f0f0) >> 4) | ((store & 0x0f0f_0f0f) << 4);
    let store = store.swap_bytes();

    store << (u32::BITS as usize - (len * 2))
}

#[inline(always)]
const fn reverse_width_3(store: u32, len: usize) -> u32 {
    reverse_fields::<3>(store, len)
}

#[inline(always)]
const fn reverse_width_4(store: u32, len: usize) -> u32 {
    let store = ((store & 0xf0f0_f0f0) >> 4) | ((store & 0x0f0f_0f0f) << 4);
    let store = store.swap_bytes();

    store << (u32::BITS as usize - (len * 4))
}

macro_rules! reverse_field_helpers {
    ($($helper:ident => $width:literal),+ $(,)?) => {
        $(
            #[inline(always)]
            const fn $helper(store: u32, len: usize) -> u32 {
                reverse_fields::<$width>(store, len)
            }
        )+
    };
}

reverse_field_helpers!(
    reverse_width_5 => 5,
    reverse_width_6 => 6,
    reverse_width_7 => 7,
);

#[inline(always)]
const fn reverse_width_8(store: u32, len: usize) -> u32 {
    store.swap_bytes() << (u32::BITS as usize - (len * 8))
}

#[inline(always)]
const fn reverse_width_9(store: u32, len: usize) -> u32 {
    reverse_two_or_three_fields::<9>(store, len)
}

#[inline(always)]
const fn reverse_width_10(store: u32, len: usize) -> u32 {
    reverse_two_or_three_fields::<10>(store, len)
}

macro_rules! reverse_two_field_helpers {
    ($($helper:ident => $width:literal),+ $(,)?) => {
        $(
            #[inline(always)]
            const fn $helper(store: u32, _len: usize) -> u32 {
                reverse_two_fields::<$width>(store)
            }
        )+
    };
}

reverse_two_field_helpers!(
    reverse_width_11 => 11,
    reverse_width_12 => 12,
    reverse_width_13 => 13,
    reverse_width_14 => 14,
    reverse_width_15 => 15,
);

#[inline(always)]
const fn reverse_width_16(store: u32, _len: usize) -> u32 {
    store.rotate_left(16)
}

#[inline(always)]
const fn reverse_noop(store: u32, _len: usize) -> u32 {
    store
}

macro_rules! impl_reverse {
    (scalar $scalar_store:ty; $($bits:literal => $helper:ident),+ $(,)?) => {
        $(
            unsafe impl Reverse<Bits<$bits>, $scalar_store> for u32
            where
                Bits<$bits>: scalar::Layout<Store = $scalar_store>,
            {
                #[inline(always)]
                unsafe fn reverse_unchecked(&mut self, len: usize) {
                    unsafe {
                        std::hint::assert_unchecked(len > 1);
                        std::hint::assert_unchecked(len <= u32::BITS as usize / $bits);
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
    9 => reverse_width_9,
    10 => reverse_width_10,
    11 => reverse_width_11,
    12 => reverse_width_12,
    13 => reverse_width_13,
    14 => reverse_width_14,
    15 => reverse_width_15,
    16 => reverse_width_16,
);

impl_reverse!(
    scalar u32;
    17 => reverse_noop,
    18 => reverse_noop,
    19 => reverse_noop,
    20 => reverse_noop,
    21 => reverse_noop,
    22 => reverse_noop,
    23 => reverse_noop,
    24 => reverse_noop,
    25 => reverse_noop,
    26 => reverse_noop,
    27 => reverse_noop,
    28 => reverse_noop,
    29 => reverse_noop,
    30 => reverse_noop,
    31 => reverse_noop,
    32 => reverse_noop,
);

#[cfg(test)]
mod tests {
    use super::{super::testing, *};

    const REVERSED_WIDTH_4: u32 = reverse_width_4(0x1234_5678, 8);

    fn assert_samples(width: usize, len: usize, helper: fn(u32, usize) -> u32) {
        testing::assert_samples(width, len, u32::BITS as usize, |store| {
            helper(store as u32, len) as u64
        });
    }

    fn assert_reverse_impl_samples<ScalarLayout, ScalarStore>()
    where
        u32: Reverse<ScalarLayout, ScalarStore>,
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store,
    {
        testing::assert_reverse_impl::<u32, ScalarLayout, ScalarStore>();

        let width = ScalarLayout::BITS.get();
        for len in 2..=(u32::BITS as usize / width) {
            testing::assert_samples(width, len, u32::BITS as usize, |store| {
                let mut store = store as u32;

                unsafe {
                    <u32 as Reverse<ScalarLayout, ScalarStore>>::reverse_unchecked(&mut store, len);
                }

                store as u64
            });
        }
    }

    macro_rules! assert_helpers {
        ($($width:literal => $helper:ident),+ $(,)?) => {
            $(
                for len in 2..=(u32::BITS as usize / $width) {
                    assert_samples($width, len, $helper);
                }
            )+
        };
    }

    #[test]
    fn helpers_are_const_usable() {
        assert_eq!(REVERSED_WIDTH_4, 0x8765_4321);
    }

    #[test]
    fn helpers_match_reference_for_valid_lengths() {
        assert_helpers!(
            1 => reverse_width_1,
            2 => reverse_width_2,
            3 => reverse_width_3,
            4 => reverse_width_4,
            5 => reverse_width_5,
            6 => reverse_width_6,
            7 => reverse_width_7,
            8 => reverse_width_8,
            9 => reverse_width_9,
            10 => reverse_width_10,
            11 => reverse_width_11,
            12 => reverse_width_12,
            13 => reverse_width_13,
            14 => reverse_width_14,
            15 => reverse_width_15,
            16 => reverse_width_16,
        );

        for width in 17..=32 {
            assert_samples(width, 1, reverse_noop);
        }
    }

    #[test]
    fn reverse_impls_cover_all_u32_scalar_widths() {
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
        assert_reverse_impl_samples::<Bits<17>, u32>();
        assert_reverse_impl_samples::<Bits<18>, u32>();
        assert_reverse_impl_samples::<Bits<19>, u32>();
        assert_reverse_impl_samples::<Bits<20>, u32>();
        assert_reverse_impl_samples::<Bits<21>, u32>();
        assert_reverse_impl_samples::<Bits<22>, u32>();
        assert_reverse_impl_samples::<Bits<23>, u32>();
        assert_reverse_impl_samples::<Bits<24>, u32>();
        assert_reverse_impl_samples::<Bits<25>, u32>();
        assert_reverse_impl_samples::<Bits<26>, u32>();
        assert_reverse_impl_samples::<Bits<27>, u32>();
        assert_reverse_impl_samples::<Bits<28>, u32>();
        assert_reverse_impl_samples::<Bits<29>, u32>();
        assert_reverse_impl_samples::<Bits<30>, u32>();
        assert_reverse_impl_samples::<Bits<31>, u32>();
        assert_reverse_impl_samples::<Bits<32>, u32>();
    }
}
