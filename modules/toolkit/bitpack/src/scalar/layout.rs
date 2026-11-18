use super::store::Store;
use std::num::NonZeroUsize;

/// Compile-time layout for a single scalar.
///
/// Compile-time generated implementations map `Bits<N>` to the smallest
/// supported scalar store and a mask covering the low `N` bits of that store.
///
/// Besides `Bits<N>`, codecs implement `Layout` through a blanket impl that
/// delegates all details to their associated `bits::Bits<N>` marker.
///
/// This allows Scalar values to carry all necessary layout information
/// and remain Codec-bound for type safety.
pub trait Layout: Copy + Send + Sync + 'static {
    type Store: Store;

    const BITS: NonZeroUsize;
    const MASK: Self::Store;
}

include!(concat!(env!("OUT_DIR"), "/bitpack_scalar_layout_impls.rs"));
