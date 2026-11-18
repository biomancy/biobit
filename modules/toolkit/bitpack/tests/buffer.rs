mod utils;

use biobit_bitpack::{Codec, PackedBuffer, scalar};
use utils::ops::{Apply, Op, Raw};

#[test]
fn empty_buffer_reports_failed_position_dependent_ops() {
    let mut buffer = PackedBuffer::<Raw<2>, u8>::new();

    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    assert!(!buffer.set(0, Raw::<2>::encode(&1u8)));
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    assert!(!buffer.slide_left(Raw::<2>::encode(&2u8)));
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    assert!(!buffer.slide_right(Raw::<2>::encode(&3u8)));
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    assert!(!buffer.drop_left());
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    assert!(!buffer.drop_right());
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    buffer.reverse();
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);

    buffer.complement();
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[]);
}

#[test]
fn pushes_respect_fixed_capacity_and_drops_update_order() {
    let mut buffer = PackedBuffer::<Raw<2>, u8>::new();

    for (op, expected) in &[
        (Op::push_right(Raw::<2>::encode(&2u8)), &vec![2]),
        (Op::push_left(Raw::<2>::encode(&3u8)), &vec![3, 2]),
        (Op::push_right(Raw::<2>::encode(&0u8)), &vec![3, 2, 0]),
        (Op::push_left(Raw::<2>::encode(&1u8)), &vec![1, 3, 2, 0]),
    ] {
        buffer = op
            .apply(buffer)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<2>, _>(&buffer, expected);
    }

    assert!(buffer.is_full());
    assert!(!buffer.push_left(Raw::<2>::encode(&2u8)));
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[1, 3, 2, 0]);
    assert!(!buffer.push_right(Raw::<2>::encode(&2u8)));
    utils::assert_symbols::<Raw<2>, _>(&buffer, &[1, 3, 2, 0]);

    for (op, expected) in &[
        (Op::drop_left(), &vec![3, 2, 0]),
        (Op::drop_right(), &vec![3, 2]),
    ] {
        buffer = op
            .apply(buffer)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<2>, _>(&buffer, expected);
    }
}

#[test]
fn set_reverse_and_complement_update_contents() {
    let initial = [0u8, 1, 2, 3, 4].map(|symbol| Raw::<3>::encode(&symbol));
    let mut buffer = PackedBuffer::<Raw<3>, u16>::try_from_slice(&initial).unwrap();

    for (op, expected) in &[
        (Op::set(0, Raw::<3>::encode(&6u8)), &vec![6, 1, 2, 3, 4]),
        (Op::set(3, Raw::<3>::encode(&0u8)), &vec![6, 1, 2, 0, 4]),
        (Op::reverse(), &vec![4, 0, 2, 1, 6]),
        (Op::complement(), &vec![3, 7, 5, 6, 1]),
    ] {
        buffer = op
            .apply(buffer)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<3>, _>(&buffer, expected);
    }

    assert!(!buffer.set(5, Raw::<3>::encode(&2u8)));
    utils::assert_symbols::<Raw<3>, _>(&buffer, &[3, 7, 5, 6, 1]);
}

#[test]
fn slides_follow_prepend_and_append_semantics() {
    let initial = [0u8, 1, 2, 3, 4].map(|symbol| Raw::<4>::encode(&symbol));
    let mut buffer = PackedBuffer::<Raw<4>, u32>::try_from_slice(&initial).unwrap();

    for (op, expected) in &[
        (
            Op::slide_left(Raw::<4>::encode(&10u8)),
            &vec![10, 0, 1, 2, 3],
        ),
        (
            Op::slide_left(Raw::<4>::encode(&11u8)),
            &vec![11, 10, 0, 1, 2],
        ),
        (
            Op::slide_right(Raw::<4>::encode(&12u8)),
            &vec![10, 0, 1, 2, 12],
        ),
        (
            Op::slide_right(Raw::<4>::encode(&13u8)),
            &vec![0, 1, 2, 12, 13],
        ),
    ] {
        buffer = op
            .apply(buffer)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<4>, _>(&buffer, expected);
    }
}

#[test]
fn mixed_sequence_crosses_word_boundaries() {
    let initial = [0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9].map(|symbol| Raw::<9>::encode(&symbol));
    let mut buffer = PackedBuffer::<Raw<9>, [u64; 2]>::try_from_slice(&initial).unwrap();

    for (op, expected) in &[
        (
            Op::set(7, Raw::<9>::encode(&31u16)),
            &vec![0, 1, 2, 3, 4, 5, 6, 31, 8, 9],
        ),
        (
            Op::push_left(Raw::<9>::encode(&32u16)),
            &vec![32, 0, 1, 2, 3, 4, 5, 6, 31, 8, 9],
        ),
        (
            Op::push_right(Raw::<9>::encode(&33u16)),
            &vec![32, 0, 1, 2, 3, 4, 5, 6, 31, 8, 9, 33],
        ),
        (
            Op::slide_right(Raw::<9>::encode(&34u16)),
            &vec![0, 1, 2, 3, 4, 5, 6, 31, 8, 9, 33, 34],
        ),
        (
            Op::slide_left(Raw::<9>::encode(&35u16)),
            &vec![35, 0, 1, 2, 3, 4, 5, 6, 31, 8, 9, 33],
        ),
        (Op::reverse(), &vec![33, 9, 8, 31, 6, 5, 4, 3, 2, 1, 0, 35]),
        (
            Op::complement(),
            &vec![478, 502, 503, 480, 505, 506, 507, 508, 509, 510, 511, 476],
        ),
        (
            Op::drop_left(),
            &vec![502, 503, 480, 505, 506, 507, 508, 509, 510, 511, 476],
        ),
        (
            Op::drop_right(),
            &vec![502, 503, 480, 505, 506, 507, 508, 509, 510, 511],
        ),
    ] {
        buffer = op
            .apply(buffer)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<9>, _>(&buffer, expected);
    }
}

macro_rules! mirror_matrix_test {
    ($name:ident, $codec:ty, $store:ty) => {
        #[test]
        fn $name() {
            const SEEDS: &[u64] = &[0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210];
            let capacity = PackedBuffer::<$codec, $store>::CAPACITY;

            utils::run_matrix_for::<$codec, PackedBuffer<$codec, $store>, _, _, _>(
                256,
                SEEDS,
                utils::bounded_boundary_lengths(<$codec as scalar::Layout>::BITS.get(), capacity),
                |scalars| PackedBuffer::try_from_slice(scalars).unwrap(),
                |rng, len| loop {
                    let op = rng.op::<$codec>(len);

                    if len < capacity || !matches!(op, Op::Grow(_)) {
                        break op;
                    }
                },
            );
        }
    };
}

mirror_matrix_test!(mirror_execution_matches_raw_1_in_u8, Raw<1>, u8);
mirror_matrix_test!(mirror_execution_matches_raw_2_in_u8, Raw<2>, u8);
mirror_matrix_test!(mirror_execution_matches_raw_3_in_u16, Raw<3>, u16);
mirror_matrix_test!(mirror_execution_matches_raw_4_in_u32, Raw<4>, u32);
mirror_matrix_test!(mirror_execution_matches_raw_8_in_u64, Raw<8>, u64);
mirror_matrix_test!(mirror_execution_matches_raw_9_in_u64, Raw<9>, u64);
mirror_matrix_test!(mirror_execution_matches_raw_9_in_words, Raw<9>, [u64; 2]);
mirror_matrix_test!(mirror_execution_matches_raw_16_in_words, Raw<16>, [u64; 2]);
mirror_matrix_test!(mirror_execution_matches_raw_31_in_words, Raw<31>, [u64; 2]);
mirror_matrix_test!(mirror_execution_matches_raw_32_in_words, Raw<32>, [u64; 2]);
mirror_matrix_test!(mirror_execution_matches_raw_63_in_words, Raw<63>, [u64; 2]);
mirror_matrix_test!(mirror_execution_matches_raw_64_in_words, Raw<64>, [u64; 2]);
