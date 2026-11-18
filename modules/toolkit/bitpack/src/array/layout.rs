use crate::{len, scalar, store};

/// Compile-time layout for a fixed-length packed scalar array.
///
/// Implementations choose the physical array store for `L` scalar slots of a
/// given `Bits<N>` width. Shape-derived values such as used bits, padding bits,
/// and masks live on `array::PackedArray`, because they depend on both width and
/// length. Codecs implement this through a blanket impl that delegates to their
/// associated `bits::Bits<N>` marker.
pub trait Layout<L: len::Length>: scalar::Layout {
    type Store: store::StaticStore;
}

include!(concat!(env!("OUT_DIR"), "/bitpack_array_layout_impls.rs"));
