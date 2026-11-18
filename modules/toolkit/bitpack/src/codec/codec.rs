use crate::len::Length;
use crate::{array, scalar};
use std::num::NonZeroUsize;

/// Type-level marker for a fixed number of bits per packed scalar.
///
/// `Bits<BITS>` carries no value. It exists so generated layout implementations
/// can map a const bit width to concrete scalar and array storage choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bits<const BITS: usize>;

/// Converts semantic symbols to and from fixed-width packed values.
///
/// A codec is the semantic layer over the generated bit layouts. `Bits` selects
/// the raw width and storage implementations, while `Scalar<Codec>` and
/// `PackedArray<Codec, L>` carry the stronger invariant that their bit-packed
/// representations are valid for this codec.
///
/// The blanket layout impls below are the key trick: Rust cannot currently use
/// `Self::BITS_PER_SYMBOL` in const-generic type positions, so each codec names
/// its generated `bits::Bits<N>` marker as an associated type instead.
pub trait Codec: Copy + Send + Sync + 'static {
    /// The semantic symbol type for this codec.
    type Symbol;

    /// One of the `bits::Bits<N>` markers
    type Bits: scalar::Layout;

    /// Converts a semantic symbol into a codec-valid scalar.
    ///
    /// This operation must be infallible for valid symbols.
    fn encode(symbol: &Self::Symbol) -> scalar::Scalar<Self>;

    /// Converts a codec-valid scalar back into a semantic symbol.
    ///
    /// This is infallible because `scalar::Scalar<Self>` is only
    /// constructed from `Self::encode`-provided scalars.
    fn decode(scalar: scalar::Scalar<Self>) -> Self::Symbol;
}

/// Codec whose semantic complement is the bitwise complement of each scalar.
///
/// # Safety
///
/// Implementors must guarantee that every masked scalar codeword is valid and
/// that `codeword ^ Self::MASK` decodes to the semantic complement.
pub unsafe trait BitComplement: Codec {}

impl<C: Codec> scalar::Layout for C {
    type Store = <C::Bits as scalar::Layout>::Store;

    const BITS: NonZeroUsize = <C::Bits as scalar::Layout>::BITS;
    const MASK: Self::Store = <C::Bits as scalar::Layout>::MASK;
}

impl<C, L> array::Layout<L> for C
where
    C: Codec,
    L: Length,
    C::Bits: array::Layout<L>,
{
    type Store = <C::Bits as array::Layout<L>>::Store;
}
