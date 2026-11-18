mod utils;

use biobit_bitpack::{BitComplement, PackedBuffer, Scalar, scalar, store};
use std::fmt::Debug;
use utils::ops::{Apply, ApplyInPlace, GrowOp, Op, Raw, ShrinkOp, TestCodec, assert_matches};
use utils::rng::TinyRng;

// Compare one operation's return value and resulting contents with a Vec model.
fn check_buffer_operation<C, S>(
    buffer: &mut PackedBuffer<C, S>,
    mirror: &mut Vec<Scalar<C>>,
    op: Op<C>,
) where
    C: BitComplement + Debug,
    S: store::StaticStore
        + store::Pack<<C as scalar::Layout>::Store>
        + store::ScalarGet<<C as scalar::Layout>::Store>
        + store::Unpack<<C as scalar::Layout>::Store>
        + store::PushLeft<<C as scalar::Layout>::Store>
        + store::PushRight<<C as scalar::Layout>::Store>
        + store::DropLeft<<C as scalar::Layout>::Store>
        + store::DropRight<<C as scalar::Layout>::Store>,
    utils::ops::InPlaceOp<C>: ApplyInPlace<PackedBuffer<C, S>>,
{
    // The model Vec can grow past the fixed buffer's capacity.
    let expected = if mirror.len() == PackedBuffer::<C, S>::CAPACITY && matches!(op, Op::Grow(_)) {
        None
    } else {
        Apply::<Vec<Scalar<C>>>::apply(op, mirror.clone())
    };

    let changed = match op {
        Op::InPlace(op) => op.apply(buffer),
        Op::Grow(GrowOp::PushLeft { scalar }) => buffer.push_left(scalar),
        Op::Grow(GrowOp::PushRight { scalar }) => buffer.push_right(scalar),
        Op::Shrink(ShrinkOp::DropLeft) => buffer.drop_left(),
        Op::Shrink(ShrinkOp::DropRight) => buffer.drop_right(),
    };
    assert_eq!(changed, expected.is_some(), "{op:?}");
    if let Some(expected) = expected {
        *mirror = expected;
    }
    assert_matches(buffer, mirror);
    assert_eq!(
        buffer.is_full(),
        mirror.len() == PackedBuffer::<C, S>::CAPACITY
    );
    // Repacking also checks padding bits that get and unpack cannot observe.
    let repacked = PackedBuffer::<C, S>::try_from_slice(mirror).unwrap();
    assert_eq!(buffer.store(), repacked.store(), "{op:?}");
}

// Try each operation independently at every length, including empty and full.
fn check_all_starting_lengths<C, S>(
    check: impl Fn(&mut PackedBuffer<C, S>, &mut Vec<Scalar<C>>, Op<C>),
) where
    C: TestCodec,
    S: store::StaticStore + store::Pack<<C as scalar::Layout>::Store>,
{
    let scalar = C::scalar(1);
    for len in 0..=PackedBuffer::<C, S>::CAPACITY {
        let mirror = C::scalars_from(1, len);
        let buffer = PackedBuffer::<C, S>::try_from_slice(&mirror).unwrap();
        for op in [
            Op::push_left(scalar),
            Op::push_right(scalar),
            Op::drop_left(),
            Op::drop_right(),
            Op::slide_left(scalar),
            Op::slide_right(scalar),
            Op::set(0, scalar),
            Op::set(len.saturating_sub(1), scalar),
            Op::set(len, scalar),
            Op::reverse(),
            Op::complement(),
        ] {
            check(&mut buffer.clone(), &mut mirror.clone(), op);
        }
    }
}

// Chain pseudo-random operations from lengths near storage and capacity boundaries.
fn check_seeded_sequences<C, S>(check: impl Fn(&mut PackedBuffer<C, S>, &mut Vec<Scalar<C>>, Op<C>))
where
    C: TestCodec,
    S: store::StaticStore
        + store::Pack<<C as scalar::Layout>::Store>
        + store::ScalarGet<<C as scalar::Layout>::Store>
        + store::Unpack<<C as scalar::Layout>::Store>,
{
    const SEEDS: &[u64] = &[0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210];
    let lengths = utils::bounded_boundary_lengths(
        <C as scalar::Layout>::BITS.get(),
        PackedBuffer::<C, S>::CAPACITY,
    );
    for len in lengths {
        for &seed in SEEDS {
            let mut mirror = C::scalars(len);
            let mut buffer = PackedBuffer::<C, S>::try_from_slice(&mirror).unwrap();
            let mut rng = TinyRng::new(seed);
            assert_matches(&buffer, &mirror);

            for _ in 0..256 {
                let op = rng.op::<C>(mirror.len());
                check(&mut buffer, &mut mirror, op);
            }
        }
    }
}

macro_rules! mirror_matrix_test {
    ($name:ident, $codec:ty, $store:ty) => {
        #[test]
        fn $name() {
            check_all_starting_lengths(check_buffer_operation::<$codec, $store>);
            check_seeded_sequences(check_buffer_operation::<$codec, $store>);
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
mirror_matrix_test!(mirror_execution_matches_raw_7_in_u8, Raw<7>, u8);
mirror_matrix_test!(mirror_execution_matches_raw_15_in_u16, Raw<15>, u16);
mirror_matrix_test!(mirror_execution_matches_raw_31_in_u32, Raw<31>, u32);
mirror_matrix_test!(mirror_execution_matches_raw_63_in_u64, Raw<63>, u64);
mirror_matrix_test!(
    mirror_execution_matches_raw_33_in_three_words,
    Raw<33>,
    [u64; 3]
);

// Full-width scalars fill a single slot in each primitive store.
mirror_matrix_test!(mirror_execution_matches_raw_8_in_u8, Raw<8>, u8);
mirror_matrix_test!(mirror_execution_matches_raw_16_in_u16, Raw<16>, u16);
mirror_matrix_test!(mirror_execution_matches_raw_32_in_u32, Raw<32>, u32);
mirror_matrix_test!(mirror_execution_matches_raw_64_in_u64, Raw<64>, u64);
