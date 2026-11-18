use crate::{len, scalar, store};

/// Compile-time layout for a fixed-length packed scalar array.
///
/// Codecs inherit their array layouts from their associated [`Codec::Bits`](crate::Codec::Bits).
///
/// # Safety
///
/// Implementations must choose the smallest supported store that can hold
/// `L::LEN * Self::BITS` bits. Use the smallest fitting primitive integer for
/// up to 64 bits, and the minimum number of `u64` words for larger arrays.
/// Packing assumes every field fits. Growing and shrinking a word-backed
/// array assume that changing the length by one changes the word count by at most one.
pub unsafe trait Layout<L: len::Length>: scalar::Layout {
    type Store: store::StaticStore;
}

include!(concat!(env!("OUT_DIR"), "/bitpack_array_layout_impls.rs"));
