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
/// [`Self::Bits`] selects the field width. Assign codewords from zero upwards.
/// A standalone [`scalar::Scalar`] holds its codeword in its least significant
/// bits. Packed sequences place the first codeword in the most significant field.
///
/// A two-bit field can also store a three-symbol alphabet.
///
/// ```
/// use biobit_bitpack::{Bits, Codec, Scalar};
///
/// #[derive(Clone, Copy)]
/// struct Trit;
///
/// // SAFETY: The valid symbols 0, 1, and 2 encode and decode as themselves.
/// unsafe impl Codec for Trit {
///     type Symbol = u8;
///     type Bits = Bits<2>;
///     const MAX_CODE: u8 = 2;
///
///     fn encode(symbol: &u8) -> Scalar<Self> {
///         Scalar::new(*symbol).unwrap()
///     }
///
///     fn decode(scalar: Scalar<Self>) -> u8 {
///         scalar.store()
///     }
/// }
///
/// for value in 0..=2 {
///     assert_eq!(Scalar::<Trit>::new(value).map(Trit::decode), Some(value));
/// }
/// assert!(Scalar::<Trit>::new(3).is_none());
/// ```
///
/// # Safety
///
/// The valid codewords must be exactly `0..=Self::MAX_CODE`, and `MAX_CODE`
/// must fit within the mask of [`Self::Bits`]. [`Self::encode`] must return
/// a codeword in this range for each valid symbol. [`Self::decode`] must accept
/// every codeword in the range. [`scalar::Scalar::new`] relies on the upper
/// bound, and unpacking uses zero to initialize scalar arrays.
pub unsafe trait Codec: Copy + Send + Sync + 'static {
    /// The semantic symbol type for this codec.
    type Symbol;

    /// Scalar layout, usually a generated [`Bits<N>`](Bits) marker.
    type Bits: scalar::Layout;

    /// Largest valid codeword, inclusive. Defaults to the full field mask.
    /// A three-symbol codec with [`Bits<2>`](Bits) sets this to `2`.
    const MAX_CODE: <Self as scalar::Layout>::Store = <Self as scalar::Layout>::MASK;

    /// Converts a semantic symbol into a codec-valid scalar.
    ///
    /// This operation must be infallible for valid symbols.
    fn encode(symbol: &Self::Symbol) -> scalar::Scalar<Self>;

    /// Converts a codec-valid scalar back into a semantic symbol.
    fn decode(scalar: scalar::Scalar<Self>) -> Self::Symbol;
}

/// Codec whose semantic complement is the bitwise complement of each scalar.
///
/// # Safety
///
/// `Self::MAX_CODE` must equal the field mask, and flipping every bit in a
/// codeword within that mask must produce its semantic complement.
pub unsafe trait BitComplement: Codec {}

// SAFETY: The store, width, and mask all come from the same scalar layout.
unsafe impl<C: Codec> scalar::Layout for C {
    type Store = <C::Bits as scalar::Layout>::Store;

    const BITS: NonZeroUsize = <C::Bits as scalar::Layout>::BITS;
    const MASK: Self::Store = <C::Bits as scalar::Layout>::MASK;
}

// SAFETY: The associated bit layout selects the store for this exact length.
unsafe impl<C, L> array::Layout<L> for C
where
    C: Codec,
    L: Length,
    C::Bits: array::Layout<L>,
{
    type Store = <C::Bits as array::Layout<L>>::Store;
}
