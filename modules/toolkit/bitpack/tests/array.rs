mod utils;

use std::vec;

use biobit_bitpack::{Codec, Len, PackedArray};
use utils::ops::{ApplyInPlace, InPlaceOp, Raw};

#[test]
fn construction_and_views_handle_partial_padding() {
    let scalars = [0u8, 1, 2, 3, 2].map(|symbol| Raw::<2>::encode(&symbol));
    let array = PackedArray::<Raw<2>, Len<5>>::from_array(&scalars);

    utils::assert_symbols::<Raw<2>, _>(&array, &[0, 1, 2, 3, 2]);
    assert_eq!(Raw::<2>::decode(array.first()), 0);
    assert_eq!(Raw::<2>::decode(array.last()), 2);
    assert_eq!(array.get(5), None);
    assert_eq!(array.get(3).map(Raw::<2>::decode), Some(3));

    assert!(PackedArray::<Raw<2>, Len<5>>::try_from_slice(&scalars[..4]).is_none());
    assert!(PackedArray::<Raw<2>, Len<5>>::try_from_slice(&scalars).is_some());

    let mut too_short = vec![Raw::<2>::encode(&0u8); 4];
    assert!(!array.copy_to_slice(&mut too_short));

    let mut out = [Raw::<2>::encode(&0u8); 5];
    assert!(array.copy_to_slice(&mut out));
    assert_eq!(out.map(Raw::<2>::decode), [0, 1, 2, 3, 2]);
}

#[test]
fn set_reverse_and_complement_update_contents() {
    let scalars = [0u8, 1, 2, 3, 4].map(|symbol| Raw::<3>::encode(&symbol));
    let mut array = PackedArray::<Raw<3>, Len<5>>::from_array(&scalars);

    for (op, expected) in [
        (
            InPlaceOp::Set {
                pos: 0,
                scalar: Raw::<3>::encode(&6u8),
            },
            [6, 1, 2, 3, 4],
        ),
        (
            InPlaceOp::Set {
                pos: 3,
                scalar: Raw::<3>::encode(&0u8),
            },
            [6, 1, 2, 0, 4],
        ),
        (InPlaceOp::Reverse, [4, 0, 2, 1, 6]),
        (InPlaceOp::Complement, [3, 7, 5, 6, 1]),
    ] {
        assert!(op.apply(&mut array));
        utils::assert_symbols::<Raw<3>, _>(&array, &expected);
    }

    assert!(!array.set(5, Raw::<3>::encode(&2u8)));
    utils::assert_symbols::<Raw<3>, _>(&array, &[3, 7, 5, 6, 1]);
}

#[test]
fn slides_update_fixed_window() {
    let scalars = [0u8, 1, 2, 3, 4].map(|symbol| Raw::<4>::encode(&symbol));
    let mut array = PackedArray::<Raw<4>, Len<5>>::from_array(&scalars);

    for (op, expected) in [
        (
            InPlaceOp::SlideLeft {
                scalar: Raw::<4>::encode(&10u8),
            },
            [10, 0, 1, 2, 3],
        ),
        (
            InPlaceOp::SlideLeft {
                scalar: Raw::<4>::encode(&11u8),
            },
            [11, 10, 0, 1, 2],
        ),
        (
            InPlaceOp::SlideRight {
                scalar: Raw::<4>::encode(&12u8),
            },
            [10, 0, 1, 2, 12],
        ),
        (
            InPlaceOp::SlideRight {
                scalar: Raw::<4>::encode(&13u8),
            },
            [0, 1, 2, 12, 13],
        ),
    ] {
        assert!(op.apply(&mut array));
        utils::assert_symbols::<Raw<4>, _>(&array, &expected);
    }
}

#[test]
fn single_slot_operations_are_well_defined() {
    let mut array = PackedArray::<Raw<9>, Len<1>>::from_array(&[Raw::<9>::encode(&0x001u16)]);

    array.reverse();
    utils::assert_symbols::<Raw<9>, _>(&array, &[0x001]);

    array.complement();
    utils::assert_symbols::<Raw<9>, _>(&array, &[0x1fe]);

    array.slide_left(Raw::<9>::encode(&0x155u16));
    utils::assert_symbols::<Raw<9>, _>(&array, &[0x155]);

    array.slide_right(Raw::<9>::encode(&0x0aau16));
    utils::assert_symbols::<Raw<9>, _>(&array, &[0x0aa]);
}

#[test]
fn pushes_and_drops_with_same_primitive_store_update_order() {
    let scalars = [0u8, 1, 2].map(|symbol| Raw::<2>::encode(&symbol));
    let array = PackedArray::<Raw<2>, Len<3>>::from_array(&scalars);

    let pushed = array.clone().push_left(Raw::<2>::encode(&3u8));
    utils::assert_symbols::<Raw<2>, _>(&pushed, &[3, 0, 1, 2]);

    let dropped = pushed.drop_right();
    utils::assert_symbols::<Raw<2>, _>(&dropped, &[3, 0, 1]);

    let pushed = array.clone().push_right(Raw::<2>::encode(&3u8));
    utils::assert_symbols::<Raw<2>, _>(&pushed, &[0, 1, 2, 3]);

    let dropped = pushed.drop_left();
    utils::assert_symbols::<Raw<2>, _>(&dropped, &[1, 2, 3]);
}

#[test]
fn pushes_and_drops_across_primitive_store_widths_update_order() {
    let array = PackedArray::<Raw<9>, Len<1>>::from_array(&[Raw::<9>::encode(&1u16)]);

    let pushed = array.clone().push_left(Raw::<9>::encode(&5u16));
    utils::assert_symbols::<Raw<9>, _>(&pushed, &[5, 1]);

    let dropped = pushed.drop_left();
    utils::assert_symbols::<Raw<9>, _>(&dropped, &[1]);

    let pushed = array.push_right(Raw::<9>::encode(&5u16));
    utils::assert_symbols::<Raw<9>, _>(&pushed, &[1, 5]);

    let dropped = pushed.drop_right();
    utils::assert_symbols::<Raw<9>, _>(&dropped, &[1]);
}

#[test]
fn grows_and_shrinks_between_u64_and_word_store() {
    let symbols: Vec<_> = (0u8..8).collect();
    let scalars: Vec<_> = symbols.iter().map(Raw::<8>::encode).collect();
    let array = PackedArray::<Raw<8>, Len<8>>::try_from_slice(&scalars).unwrap();

    let pushed = array.clone().push_left(Raw::<8>::encode(&200u8));
    utils::assert_symbols::<Raw<8>, _>(&pushed, &[200, 0, 1, 2, 3, 4, 5, 6, 7]);

    let dropped = pushed.drop_right();
    utils::assert_symbols::<Raw<8>, _>(&dropped, &[200, 0, 1, 2, 3, 4, 5, 6]);

    let pushed = array.push_right(Raw::<8>::encode(&201u8));
    utils::assert_symbols::<Raw<8>, _>(&pushed, &[0, 1, 2, 3, 4, 5, 6, 7, 201]);

    let dropped = pushed.drop_left();
    utils::assert_symbols::<Raw<8>, _>(&dropped, &[1, 2, 3, 4, 5, 6, 7, 201]);
}

#[test]
fn word_backed_in_place_ops_cross_word_boundaries() {
    let scalars: Vec<_> = (0u16..10).map(|symbol| Raw::<9>::encode(&symbol)).collect();
    let mut array = PackedArray::<Raw<9>, Len<10>>::try_from_slice(&scalars).unwrap();

    for (op, expected) in [
        (
            InPlaceOp::Set {
                pos: 7,
                scalar: Raw::<9>::encode(&31u16),
            },
            [0, 1, 2, 3, 4, 5, 6, 31, 8, 9],
        ),
        (InPlaceOp::Reverse, [9, 8, 31, 6, 5, 4, 3, 2, 1, 0]),
        (
            InPlaceOp::Complement,
            [502, 503, 480, 505, 506, 507, 508, 509, 510, 511],
        ),
        (
            InPlaceOp::SlideRight {
                scalar: Raw::<9>::encode(&34u16),
            },
            [503, 480, 505, 506, 507, 508, 509, 510, 511, 34],
        ),
        (
            InPlaceOp::SlideLeft {
                scalar: Raw::<9>::encode(&35u16),
            },
            [35, 503, 480, 505, 506, 507, 508, 509, 510, 511],
        ),
    ] {
        assert!(op.apply(&mut array));
        utils::assert_symbols::<Raw<9>, _>(&array, &expected);
    }
}

#[test]
fn word_backed_grows_and_shrinks_across_word_counts() {
    let symbols: Vec<_> = (0..64).map(|pos| (pos & 3) as u8).collect();
    let scalars: Vec<_> = symbols.iter().map(Raw::<2>::encode).collect();
    let array = PackedArray::<Raw<2>, Len<64>>::try_from_slice(&scalars).unwrap();

    let mut expected = symbols.clone();
    expected.insert(0, 3);
    let pushed = array.clone().push_left(Raw::<2>::encode(&3u8));
    utils::assert_symbols::<Raw<2>, _>(&pushed, &expected);

    let dropped = pushed.drop_left();
    utils::assert_symbols::<Raw<2>, _>(&dropped, &symbols);

    let mut expected = symbols.clone();
    expected.push(2);
    let pushed = array.push_right(Raw::<2>::encode(&2u8));
    utils::assert_symbols::<Raw<2>, _>(&pushed, &expected);

    let dropped = pushed.drop_right();
    utils::assert_symbols::<Raw<2>, _>(&dropped, &symbols);
}

#[test]
fn word_backed_arrays_can_shrink_to_u64() {
    let symbols: Vec<_> = (0..32).map(|pos| (pos & 3) as u8).chain([3]).collect();
    let scalars: Vec<_> = symbols.iter().map(Raw::<2>::encode).collect();
    let array = PackedArray::<Raw<2>, Len<33>>::try_from_slice(&scalars).unwrap();

    // The final scalar occupies the highest two bits of the second word.
    let first_word = 0x1b1b_1b1b_1b1b_1b1b;
    assert_eq!(*array.store(), [first_word, 0b11 << 62]);

    let dropped = array.clone().drop_left();
    utils::assert_symbols::<Raw<2>, _>(&dropped, &symbols[1..]);

    let dropped = array.drop_right();
    utils::assert_symbols::<Raw<2>, _>(&dropped, &symbols[..32]);
    assert_eq!(*dropped.store(), first_word);
}

#[test]
fn drop_right_then_append_uses_the_new_scalar() {
    let scalars = [0, 0, 7].map(|value| Raw::<3>::encode(&value));
    let array = PackedArray::<_, Len<3>>::from_array(&scalars);
    // Narrowing turns part of the removed field into low-order padding.
    let dropped = array.drop_right();
    let result = dropped.clone().push_right(Raw::<3>::encode(&0));

    assert_eq!(result.to_array().map(Raw::<3>::decode), [0, 0, 0]);
    assert_eq!(*dropped.store(), 0);
}

#[test]
fn drop_right_to_u64_clears_padding() {
    let scalars = [0, 0, 0, 0, 0, 0, 0, 511].map(|value| Raw::<9>::encode(&value));
    let array = PackedArray::<_, Len<8>>::from_array(&scalars);
    let dropped = array.drop_right();

    // The removed scalar straddled the word boundary, leaving one padding bit.
    utils::assert_symbols::<Raw<9>, _>(&dropped, &[0; 7]);
    assert_eq!(*dropped.store(), 0);
}
