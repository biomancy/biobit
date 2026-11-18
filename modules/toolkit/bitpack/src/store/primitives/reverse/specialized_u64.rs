use super::Reverse;
use crate::{Bits, scalar};

#[inline(always)]
const fn scalar_mask<const WIDTH: usize>() -> u64 {
    if WIDTH == u64::BITS as usize {
        u64::MAX
    } else {
        (1u64 << WIDTH) - 1
    }
}

#[inline(always)]
const fn reverse_fields<const WIDTH: usize>(store: u64, len: usize) -> u64 {
    let mask = scalar_mask::<WIDTH>();
    let mut reversed = 0;
    let mut pos = 0;

    while pos < len {
        let src_shift = u64::BITS as usize - ((pos + 1) * WIDTH);
        let scalar = (store >> src_shift) & mask;
        let dst_shift = u64::BITS as usize - ((len - pos) * WIDTH);

        reversed |= scalar << dst_shift;
        pos += 1;
    }

    reversed
}

#[inline(always)]
const fn reverse_two_fields<const WIDTH: usize>(store: u64) -> u64 {
    let mask = scalar_mask::<WIDTH>();
    let high = mask << (u64::BITS as usize - WIDTH);
    let low = mask << (u64::BITS as usize - (WIDTH * 2));

    ((store & high) >> WIDTH) | ((store & low) << WIDTH)
}

#[inline(always)]
const fn reverse_three_fields<const WIDTH: usize>(store: u64) -> u64 {
    let mask = scalar_mask::<WIDTH>();
    let high = mask << (u64::BITS as usize - WIDTH);
    let middle = mask << (u64::BITS as usize - (WIDTH * 2));
    let low = mask << (u64::BITS as usize - (WIDTH * 3));

    ((store & high) >> (WIDTH * 2)) | (store & middle) | ((store & low) << (WIDTH * 2))
}

#[inline(always)]
const fn reverse_two_or_three_fields<const WIDTH: usize>(store: u64, len: usize) -> u64 {
    match len {
        2 => reverse_two_fields::<WIDTH>(store),
        3 => reverse_three_fields::<WIDTH>(store),
        _ => store,
    }
}

#[inline(always)]
const fn reverse_width_1(store: u64, len: usize) -> u64 {
    store.reverse_bits() << (u64::BITS as usize - len)
}

#[inline(always)]
const fn reverse_width_2(store: u64, len: usize) -> u64 {
    let store = ((store & 0xcccc_cccc_cccc_cccc) >> 2) | ((store & 0x3333_3333_3333_3333) << 2);
    let store = ((store & 0xf0f0_f0f0_f0f0_f0f0) >> 4) | ((store & 0x0f0f_0f0f_0f0f_0f0f) << 4);
    let store = store.swap_bytes();

    store << (u64::BITS as usize - (len * 2))
}

#[inline(always)]
const fn reverse_width_3(store: u64, len: usize) -> u64 {
    reverse_fields::<3>(store, len)
}

#[inline(always)]
const fn reverse_width_4(store: u64, len: usize) -> u64 {
    let store = ((store & 0xf0f0_f0f0_f0f0_f0f0) >> 4) | ((store & 0x0f0f_0f0f_0f0f_0f0f) << 4);
    let store = store.swap_bytes();

    store << (u64::BITS as usize - (len * 4))
}

macro_rules! reverse_field_helpers {
    ($($helper:ident => $width:literal),+ $(,)?) => {
        $(
            #[inline(always)]
            const fn $helper(store: u64, len: usize) -> u64 {
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
const fn reverse_width_8(store: u64, len: usize) -> u64 {
    store.swap_bytes() << (u64::BITS as usize - (len * 8))
}

reverse_field_helpers!(
    reverse_width_9 => 9,
    reverse_width_10 => 10,
    reverse_width_11 => 11,
    reverse_width_12 => 12,
    reverse_width_13 => 13,
    reverse_width_14 => 14,
    reverse_width_15 => 15,
);

#[inline(always)]
const fn reverse_width_16(store: u64, len: usize) -> u64 {
    let store = ((store & 0xffff_0000_ffff_0000) >> 16) | ((store & 0x0000_ffff_0000_ffff) << 16);
    let store = store.rotate_left(32);

    store << (u64::BITS as usize - (len * 16))
}

#[inline(always)]
const fn reverse_width_17(store: u64, len: usize) -> u64 {
    reverse_two_or_three_fields::<17>(store, len)
}

#[inline(always)]
const fn reverse_width_18(store: u64, len: usize) -> u64 {
    reverse_two_or_three_fields::<18>(store, len)
}

#[inline(always)]
const fn reverse_width_19(store: u64, len: usize) -> u64 {
    reverse_two_or_three_fields::<19>(store, len)
}

#[inline(always)]
const fn reverse_width_20(store: u64, len: usize) -> u64 {
    reverse_two_or_three_fields::<20>(store, len)
}

#[inline(always)]
const fn reverse_width_21(store: u64, len: usize) -> u64 {
    reverse_two_or_three_fields::<21>(store, len)
}

macro_rules! reverse_two_field_helpers {
    ($($helper:ident => $width:literal),+ $(,)?) => {
        $(
            #[inline(always)]
            const fn $helper(store: u64, _len: usize) -> u64 {
                reverse_two_fields::<$width>(store)
            }
        )+
    };
}

reverse_two_field_helpers!(
    reverse_width_22 => 22,
    reverse_width_23 => 23,
    reverse_width_24 => 24,
    reverse_width_25 => 25,
    reverse_width_26 => 26,
    reverse_width_27 => 27,
    reverse_width_28 => 28,
    reverse_width_29 => 29,
    reverse_width_30 => 30,
    reverse_width_31 => 31,
);

#[inline(always)]
const fn reverse_width_32(store: u64, _len: usize) -> u64 {
    store.rotate_left(32)
}

#[inline(always)]
const fn reverse_noop(store: u64, _len: usize) -> u64 {
    store
}

macro_rules! impl_reverse {
    (scalar $scalar_store:ty; $($bits:literal => $helper:ident),+ $(,)?) => {
        $(
            unsafe impl Reverse<Bits<$bits>, $scalar_store> for u64
            where
                Bits<$bits>: scalar::Layout<Store = $scalar_store>,
            {
                #[inline(always)]
                unsafe fn reverse_unchecked(&mut self, len: usize) {
                    unsafe {
                        std::hint::assert_unchecked(len > 1);
                        std::hint::assert_unchecked(len <= u64::BITS as usize / $bits);
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
    17 => reverse_width_17,
    18 => reverse_width_18,
    19 => reverse_width_19,
    20 => reverse_width_20,
    21 => reverse_width_21,
    22 => reverse_width_22,
    23 => reverse_width_23,
    24 => reverse_width_24,
    25 => reverse_width_25,
    26 => reverse_width_26,
    27 => reverse_width_27,
    28 => reverse_width_28,
    29 => reverse_width_29,
    30 => reverse_width_30,
    31 => reverse_width_31,
    32 => reverse_width_32,
);

impl_reverse!(
    scalar u64;
    33 => reverse_noop,
    34 => reverse_noop,
    35 => reverse_noop,
    36 => reverse_noop,
    37 => reverse_noop,
    38 => reverse_noop,
    39 => reverse_noop,
    40 => reverse_noop,
    41 => reverse_noop,
    42 => reverse_noop,
    43 => reverse_noop,
    44 => reverse_noop,
    45 => reverse_noop,
    46 => reverse_noop,
    47 => reverse_noop,
    48 => reverse_noop,
    49 => reverse_noop,
    50 => reverse_noop,
    51 => reverse_noop,
    52 => reverse_noop,
    53 => reverse_noop,
    54 => reverse_noop,
    55 => reverse_noop,
    56 => reverse_noop,
    57 => reverse_noop,
    58 => reverse_noop,
    59 => reverse_noop,
    60 => reverse_noop,
    61 => reverse_noop,
    62 => reverse_noop,
    63 => reverse_noop,
    64 => reverse_noop,
);

#[cfg(test)]
mod tests {
    use super::{super::testing, *};

    const REVERSED_WIDTH_8: u64 = reverse_width_8(0x0102_0304_0506_0708, 8);
    const REVERSED_WIDTH_16: u64 = reverse_width_16(0x1111_2222_3333_4444, 4);

    fn assert_samples(width: usize, len: usize, helper: fn(u64, usize) -> u64) {
        testing::assert_samples(width, len, u64::BITS as usize, |store| helper(store, len));
    }

    fn assert_reverse_impl_samples<ScalarLayout, ScalarStore>()
    where
        u64: Reverse<ScalarLayout, ScalarStore>,
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        ScalarStore: scalar::Store,
    {
        testing::assert_reverse_impl::<u64, ScalarLayout, ScalarStore>();

        let width = ScalarLayout::BITS.get();
        for len in 2..=(u64::BITS as usize / width) {
            testing::assert_samples(width, len, u64::BITS as usize, |store| {
                let mut store = store;

                unsafe {
                    <u64 as Reverse<ScalarLayout, ScalarStore>>::reverse_unchecked(&mut store, len);
                }

                store
            });
        }
    }

    macro_rules! assert_helpers {
        ($($width:literal => $helper:ident),+ $(,)?) => {
            $(
                for len in 2..=(u64::BITS as usize / $width) {
                    assert_samples($width, len, $helper);
                }
            )+
        };
    }

    #[test]
    fn helpers_are_const_usable() {
        assert_eq!(REVERSED_WIDTH_8, 0x0807_0605_0403_0201);
        assert_eq!(REVERSED_WIDTH_16, 0x4444_3333_2222_1111);
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
            17 => reverse_width_17,
            18 => reverse_width_18,
            19 => reverse_width_19,
            20 => reverse_width_20,
            21 => reverse_width_21,
            22 => reverse_width_22,
            23 => reverse_width_23,
            24 => reverse_width_24,
            25 => reverse_width_25,
            26 => reverse_width_26,
            27 => reverse_width_27,
            28 => reverse_width_28,
            29 => reverse_width_29,
            30 => reverse_width_30,
            31 => reverse_width_31,
            32 => reverse_width_32,
        );

        for width in 33..=64 {
            assert_samples(width, 1, reverse_noop);
        }
    }

    #[test]
    fn reverse_impls_cover_all_u64_scalar_widths() {
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
        assert_reverse_impl_samples::<Bits<33>, u64>();
        assert_reverse_impl_samples::<Bits<34>, u64>();
        assert_reverse_impl_samples::<Bits<35>, u64>();
        assert_reverse_impl_samples::<Bits<36>, u64>();
        assert_reverse_impl_samples::<Bits<37>, u64>();
        assert_reverse_impl_samples::<Bits<38>, u64>();
        assert_reverse_impl_samples::<Bits<39>, u64>();
        assert_reverse_impl_samples::<Bits<40>, u64>();
        assert_reverse_impl_samples::<Bits<41>, u64>();
        assert_reverse_impl_samples::<Bits<42>, u64>();
        assert_reverse_impl_samples::<Bits<43>, u64>();
        assert_reverse_impl_samples::<Bits<44>, u64>();
        assert_reverse_impl_samples::<Bits<45>, u64>();
        assert_reverse_impl_samples::<Bits<46>, u64>();
        assert_reverse_impl_samples::<Bits<47>, u64>();
        assert_reverse_impl_samples::<Bits<48>, u64>();
        assert_reverse_impl_samples::<Bits<49>, u64>();
        assert_reverse_impl_samples::<Bits<50>, u64>();
        assert_reverse_impl_samples::<Bits<51>, u64>();
        assert_reverse_impl_samples::<Bits<52>, u64>();
        assert_reverse_impl_samples::<Bits<53>, u64>();
        assert_reverse_impl_samples::<Bits<54>, u64>();
        assert_reverse_impl_samples::<Bits<55>, u64>();
        assert_reverse_impl_samples::<Bits<56>, u64>();
        assert_reverse_impl_samples::<Bits<57>, u64>();
        assert_reverse_impl_samples::<Bits<58>, u64>();
        assert_reverse_impl_samples::<Bits<59>, u64>();
        assert_reverse_impl_samples::<Bits<60>, u64>();
        assert_reverse_impl_samples::<Bits<61>, u64>();
        assert_reverse_impl_samples::<Bits<62>, u64>();
        assert_reverse_impl_samples::<Bits<63>, u64>();
        assert_reverse_impl_samples::<Bits<64>, u64>();
    }
}
