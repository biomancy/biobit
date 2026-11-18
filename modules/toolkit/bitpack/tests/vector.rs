mod utils;

use std::vec;

use biobit_bitpack::{Codec, PackedVec};
use utils::ops::Raw;

use crate::utils::ops::{Apply, Op};

#[test]
fn empty_vector_reports_failed_position_dependent_ops() {
    let mut vector = PackedVec::<Raw<2>>::new();

    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    assert!(!vector.set(0, Raw::<2>::encode(&1u8)));
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    assert!(!vector.slide_left(Raw::<2>::encode(&2u8)));
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    assert!(!vector.slide_right(Raw::<2>::encode(&3u8)));
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    assert!(!vector.drop_left());
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    assert!(!vector.drop_right());
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    vector.reverse();
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    vector.complement();
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);
}

#[test]
fn pushes_and_drops_update_order() {
    let mut vector = PackedVec::<Raw<2>>::new();
    assert!(!vector.set(0, Raw::<2>::encode(&1u8)));
    utils::assert_symbols::<Raw<2>, _>(&vector, &[]);

    for (op, expected) in &[
        (Op::push_right(Raw::<2>::encode(&2u8)), &vec![2]),
        (Op::push_left(Raw::<2>::encode(&3u8)), &vec![3, 2]),
        (Op::push_right(Raw::<2>::encode(&0u8)), &vec![3, 2, 0]),
    ] {
        vector = op
            .apply(vector)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<2>, _>(&vector, expected);
    }
}

#[test]
fn set_reverse_and_complement_update_contents() {
    let initial = [0u8, 1, 2, 3, 4, 5].map(|symbol| Raw::<3>::encode(&symbol));
    let mut vector = PackedVec::from_slice(&initial);

    for (op, expected) in &[
        (Op::set(0, Raw::<3>::encode(&6u8)), &vec![6, 1, 2, 3, 4, 5]),
        (Op::set(3, Raw::<3>::encode(&0u8)), &vec![6, 1, 2, 0, 4, 5]),
        (Op::set(5, Raw::<3>::encode(&7u8)), &vec![6, 1, 2, 0, 4, 7]),
        (Op::reverse(), &vec![7, 4, 0, 2, 1, 6]),
        (Op::complement(), &vec![0, 3, 7, 5, 6, 1]),
        (Op::reverse(), &vec![1, 6, 5, 7, 3, 0]),
    ] {
        vector = op
            .apply(vector)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<3>, _>(&vector, expected);
    }

    assert!(!vector.set(6, Raw::<3>::encode(&2u8)));
    utils::assert_symbols::<Raw<3>, _>(&vector, &[1, 6, 5, 7, 3, 0]);
}

#[test]
fn slides_follow_prepend_and_append_semantics() {
    let initial = [0u8, 1, 2, 3, 4].map(|symbol| Raw::<4>::encode(&symbol));
    let mut vector = PackedVec::from_slice(&initial);

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
        vector = op
            .apply(vector)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<4>, _>(&vector, expected);
    }
}

#[test]
fn mixed_sequence_crosses_word_boundaries() {
    let initial = [0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9].map(|symbol| Raw::<9>::encode(&symbol));
    let mut vector = PackedVec::from_slice(&initial);

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
        vector = op
            .apply(vector)
            .expect(&format!("Op {op:?} should succeed"));
        utils::assert_symbols::<Raw<9>, _>(&vector, expected);
    }
}

macro_rules! mirror_matrix_test {
    ($name:ident, $bits:literal) => {
        #[test]
        fn $name() {
            const SEEDS: &[u64] = &[0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210];

            utils::run_matrix_for::<Raw<$bits>, PackedVec<Raw<$bits>>, _, _, _>(
                256,
                SEEDS,
                utils::boundary_lengths($bits),
                PackedVec::from_slice,
                |rng, len| rng.op(len),
            );
        }
    };
}

mirror_matrix_test!(mirror_execution_matches_raw_1, 1);
mirror_matrix_test!(mirror_execution_matches_raw_2, 2);
mirror_matrix_test!(mirror_execution_matches_raw_3, 3);
mirror_matrix_test!(mirror_execution_matches_raw_4, 4);
mirror_matrix_test!(mirror_execution_matches_raw_8, 8);
mirror_matrix_test!(mirror_execution_matches_raw_9, 9);
mirror_matrix_test!(mirror_execution_matches_raw_16, 16);
mirror_matrix_test!(mirror_execution_matches_raw_31, 31);
mirror_matrix_test!(mirror_execution_matches_raw_32, 32);
mirror_matrix_test!(mirror_execution_matches_raw_63, 63);
mirror_matrix_test!(mirror_execution_matches_raw_64, 64);
