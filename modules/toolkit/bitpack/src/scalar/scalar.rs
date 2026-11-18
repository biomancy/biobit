use super::{layout, store};
use crate::codec::Codec;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A single codec-valid scalar stored in the low bits of its backing integer.
///
/// `Scalar<C>` has two value-level invariants: every bit outside `C`'s scalar
/// layout mask is zero, and the remaining bits are valid for codec `C`.
#[repr(transparent)]
pub struct Scalar<C: Codec>
where
    C: layout::Layout,
{
    store: <C as layout::Layout>::Store,
}

impl<C: Codec> Copy for Scalar<C> where C: layout::Layout {}

impl<C: Codec> Clone for Scalar<C>
where
    C: layout::Layout,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: Codec> fmt::Debug for Scalar<C>
where
    C: layout::Layout,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Scalar").field(&self.store).finish()
    }
}

impl<C: Codec> PartialEq for Scalar<C>
where
    C: layout::Layout,
{
    fn eq(&self, other: &Self) -> bool {
        self.store == other.store
    }
}

impl<C: Codec> Eq for Scalar<C> where C: layout::Layout {}

impl<C: Codec> Hash for Scalar<C>
where
    C: layout::Layout,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.store.hash(state);
    }
}

impl<C: Codec> PartialOrd for Scalar<C>
where
    C: layout::Layout,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Codec> Ord for Scalar<C>
where
    C: layout::Layout,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.store.cmp(&other.store)
    }
}

impl<C: Codec> Scalar<C>
where
    C: layout::Layout,
{
    #[inline(always)]
    pub fn new(store: <C as layout::Layout>::Store) -> Option<Self> {
        (store & !<C as layout::Layout>::MASK
            == <<C as layout::Layout>::Store as store::Store>::ZERO)
            .then_some(Self { store })
    }

    /// Creates a scalar without checking the layout or codec invariant.
    ///
    /// # Safety
    ///
    /// `store` must not contain any set bit outside the mask defined by `C`.
    /// The codeword must also be valid for codec `C`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(store: <C as layout::Layout>::Store) -> Self {
        Self { store }
    }

    #[inline(always)]
    pub const fn store(self) -> <C as layout::Layout>::Store {
        self.store
    }

    #[inline(always)]
    pub(crate) fn as_store_slice(scalars: &[Self]) -> &[<C as layout::Layout>::Store] {
        // SAFETY: `Scalar<C>` is `repr(transparent)` over its scalar store.
        unsafe {
            std::slice::from_raw_parts(
                scalars.as_ptr().cast::<<C as layout::Layout>::Store>(),
                scalars.len(),
            )
        }
    }

    /// Views scalars as mutable scalar stores.
    ///
    /// # Safety
    ///
    /// Callers must not write values that violate `Scalar<C>` invariants.
    #[inline(always)]
    pub(crate) unsafe fn as_store_slice_mut_unchecked(
        scalars: &mut [Self],
    ) -> &mut [<C as layout::Layout>::Store] {
        // SAFETY: `Scalar<C>` is `repr(transparent)` over its scalar store.
        unsafe {
            std::slice::from_raw_parts_mut(
                scalars.as_mut_ptr().cast::<<C as layout::Layout>::Store>(),
                scalars.len(),
            )
        }
    }
}
