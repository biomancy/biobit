#![allow(dead_code)]

#[path = "rng.rs"]
pub mod rng;

pub use rng::ops;

use biobit_bitpack::{BitComplement, Codec, Scalar};
use rng::TinyRng;
use rng::ops::{
    Apply, ApplyInPlace, ApplyResize, GrowOp, InPlaceOp, Op, PackedView, ShrinkOp, TestCodec,
    assert_matches,
};

pub fn assert_symbols<C, P>(packed: &P, expected: &[C::Symbol])
where
    C: Codec,
    C::Symbol: Copy,
    P: PackedView<C>,
{
    let expected: Vec<_> = expected.iter().map(C::encode).collect();

    assert_matches(packed, &expected);
}

pub fn apply_pair<C, T>(packed: &mut T, mirror: &mut Vec<Scalar<C>>, operation: Op<C>) -> bool
where
    C: BitComplement,
    T: Clone + PackedView<C>,
    InPlaceOp<C>: ApplyInPlace<T> + ApplyInPlace<Vec<Scalar<C>>>,
    GrowOp<C>: ApplyResize<T, Output = T> + ApplyResize<Vec<Scalar<C>>, Output = Vec<Scalar<C>>>,
    ShrinkOp: ApplyResize<T, Output = T> + ApplyResize<Vec<Scalar<C>>, Output = Vec<Scalar<C>>>,
    Op<C>: Apply<T, Output = T> + Apply<Vec<Scalar<C>>, Output = Vec<Scalar<C>>>,
{
    let packed_next = Apply::apply(operation, packed.clone());
    let mirror_next = Apply::apply(operation, mirror.clone());
    let changed = packed_next.is_some();

    assert_eq!(changed, mirror_next.is_some());

    if let Some(next) = packed_next {
        *packed = next;
    }

    if let Some(next) = mirror_next {
        *mirror = next;
    }

    assert_matches(packed, mirror);

    changed
}

pub fn boundary_lengths(bits: usize) -> Vec<usize> {
    assert!((1..=u64::BITS as usize).contains(&bits));

    let per_word = (u64::BITS as usize / bits).max(1);
    let mut lengths = Vec::new();

    for boundary in [per_word, per_word * 2, per_word * 4] {
        lengths.push(boundary.saturating_sub(1));
        lengths.push(boundary);
        lengths.push(boundary + 1);
    }

    lengths.push(0);
    lengths.push(1);
    lengths.sort_unstable();
    lengths.dedup();

    lengths
}

pub fn bounded_boundary_lengths(bits: usize, capacity: usize) -> Vec<usize> {
    let mut lengths = boundary_lengths(bits);

    lengths.push(capacity.saturating_sub(1));
    lengths.push(capacity);
    lengths.retain(|&len| len <= capacity);
    lengths.sort_unstable();
    lengths.dedup();

    lengths
}

pub fn run_matrix_for<C, T, F, I, G>(
    steps: usize,
    seeds: &[u64],
    lengths: I,
    build: F,
    mut operation: G,
) where
    C: BitComplement + TestCodec,
    T: Clone + PackedView<C>,
    I: IntoIterator<Item = usize>,
    F: Fn(&[Scalar<C>]) -> T,
    G: FnMut(&mut TinyRng, usize) -> Op<C>,
    InPlaceOp<C>: ApplyInPlace<T> + ApplyInPlace<Vec<Scalar<C>>>,
    GrowOp<C>: ApplyResize<T, Output = T> + ApplyResize<Vec<Scalar<C>>, Output = Vec<Scalar<C>>>,
    ShrinkOp: ApplyResize<T, Output = T> + ApplyResize<Vec<Scalar<C>>, Output = Vec<Scalar<C>>>,
    Op<C>: Apply<T, Output = T> + Apply<Vec<Scalar<C>>, Output = Vec<Scalar<C>>>,
{
    for len in lengths {
        for &seed in seeds {
            let mut mirror = C::scalars(len);
            let mut packed = build(&mirror);
            let mut rng = TinyRng::new(seed);

            assert_matches(&packed, &mirror);

            for _ in 0..steps {
                let operation = operation(&mut rng, mirror.len());

                apply_pair(&mut packed, &mut mirror, operation);
            }
        }
    }
}
