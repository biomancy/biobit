use super::Reverse;
use crate::scalar;

const SAMPLE_COUNT: u64 = 512;

pub(crate) fn low_mask(bits: usize) -> u64 {
    if bits == u64::BITS as usize {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

fn canonical_store(value: u64, store_bits: usize, width: usize, len: usize) -> u64 {
    let used = width * len;

    (value & low_mask(used)) << (store_bits - used)
}

fn splitmix64(value: u64) -> u64 {
    let mut value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);

    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);

    value ^ (value >> 31)
}

pub(crate) fn reverse_reference(store: u64, store_bits: usize, width: usize, len: usize) -> u64 {
    let mut reversed = 0;
    let mask = low_mask(width);

    for src_pos in 0..len {
        let src_shift = store_bits - ((src_pos + 1) * width);
        let scalar = (store >> src_shift) & mask;
        let dst_pos = len - src_pos - 1;
        let dst_shift = store_bits - ((dst_pos + 1) * width);

        reversed |= scalar << dst_shift;
    }

    reversed
}

pub(crate) fn assert_reverse_impl<Store, ScalarLayout, ScalarStore>()
where
    Store: Reverse<ScalarLayout, ScalarStore>,
    ScalarLayout: scalar::Layout<Store = ScalarStore>,
    ScalarStore: scalar::Store,
{
}

pub(crate) fn assert_samples(
    width: usize,
    len: usize,
    store_bits: usize,
    mut reverse: impl FnMut(u64) -> u64,
) {
    let used = width * len;
    let all_active_bits = low_mask(used);

    assert_sample(width, len, store_bits, 0, &mut reverse);
    assert_sample(width, len, store_bits, all_active_bits, &mut reverse);

    for bit in 0..used {
        let one = 1u64 << bit;

        assert_sample(width, len, store_bits, one, &mut reverse);
        assert_sample(width, len, store_bits, all_active_bits ^ one, &mut reverse);
    }

    for sample in 0..SAMPLE_COUNT {
        assert_sample(
            width,
            len,
            store_bits,
            splitmix64(sample ^ ((width as u64) << 32) ^ len as u64),
            &mut reverse,
        );
    }
}

fn assert_sample(
    width: usize,
    len: usize,
    store_bits: usize,
    active_value: u64,
    reverse: &mut impl FnMut(u64) -> u64,
) {
    let store = canonical_store(active_value, store_bits, width, len);
    let expected = reverse_reference(store, store_bits, width, len);

    assert_eq!(reverse(store), expected);
}
