use super::store::Store;
use std::num::NonZeroUsize;

/// Compile-time layout for a single scalar.
///
/// The generated [`Bits<N>`](crate::Bits) layouts use the smallest scalar store
/// that fits `N` bits. Codecs inherit the layout of their [`Codec::Bits`](crate::Codec::Bits).
///
/// # Safety
///
/// `BITS` must be in `1..=64` and fit in `Store`. `MASK` must have exactly
/// the lowest `BITS` bits set. Packed writes rely on this mask to keep each
/// codeword within its field.
pub unsafe trait Layout: Copy + Send + Sync + 'static {
    type Store: Store;

    const BITS: NonZeroUsize;
    const MASK: Self::Store;
}

include!(concat!(env!("OUT_DIR"), "/bitpack_scalar_layout_impls.rs"));
