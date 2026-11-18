use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, ShlAssign, Shr, ShrAssign};

mod sealed {
    pub trait Sealed {}
}

/// Primitive unsigned storage for a single scalar code.
///
/// `Store` types are raw integer backends, not checked values. The
/// zero-padding invariant over the numeric value is enforced by
/// `scalar::Scalar`, not by the primitive integer itself.
pub trait Store:
    sealed::Sealed
    + Copy
    + Eq
    + Ord
    + Debug
    + Hash
    + Send
    + Sync
    + 'static
    + Shl<usize, Output = Self>
    + ShlAssign<usize>
    + Shr<usize, Output = Self>
    + ShrAssign<usize>
    + BitOr<Output = Self>
    + BitOrAssign
    + BitAnd<Output = Self>
    + BitAndAssign
    + Not<Output = Self>
{
    const ZERO: Self;
    const CAPACITY_BITS: NonZeroUsize;

    /// Truncates a `u64` to the store width and applies the scalar layout mask.
    ///
    /// Packed reads may pass extra high bits from adjacent fields.
    #[inline(always)]
    fn from_u64<ScalarLayout>(word: u64) -> Self
    where
        ScalarLayout: super::Layout<Store = Self>,
    {
        (unsafe { Self::from_u64_unchecked(word) }) & ScalarLayout::MASK
    }

    /// Converts a `u64` by keeping the low bits that fit in `Self`.
    ///
    /// This truncates like an integer cast, without applying a scalar layout mask.
    ///
    /// # Safety
    ///
    /// If the result is used as a scalar directly, the retained bits must
    /// satisfy that scalar's layout invariants.
    unsafe fn from_u64_unchecked(word: u64) -> Self;
}

macro_rules! impl_scalar_store {
    ($($store:ty),+ $(,)?) => {
        $(
            impl sealed::Sealed for $store {}

            impl Store for $store {
                const ZERO: Self = 0;
                const CAPACITY_BITS: NonZeroUsize =
                    NonZeroUsize::new(<$store>::BITS as usize).unwrap();

                #[inline(always)]
                unsafe fn from_u64_unchecked(word: u64) -> Self {
                    word as Self
                }
            }
        )+
    };
}

impl_scalar_store!(u8, u16, u32, u64);
