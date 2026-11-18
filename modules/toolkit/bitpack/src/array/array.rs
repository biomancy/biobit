use super::layout;
use crate::{BitComplement, Codec, len, scalar, store};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use store::{
    Complement, GrowLeft, GrowRight, Pack, Reverse, ScalarGet, ScalarSet, ShrinkLeft, ShrinkRight,
    Slide, StaticStore, Unpack,
};

/// Fixed-length array of codec-valid packed scalars.
///
/// `PackedArray<C, L>` stores `L::LEN` scalars for codec `C` in the abstract numeric
/// value of its backing store. The first scalar occupies the most significant
/// used bits, following scalars occupy progressively less significant bits,
/// and every unused low-order padding bit after `USED_BITS` is zero. This
/// defines value-level bit order only; it does not define native byte order or
/// raw memory serialization.
#[repr(transparent)]
pub struct PackedArray<C: Codec, L: len::Length>
where
    C: layout::Layout<L>,
{
    store: <C as layout::Layout<L>>::Store,
}

impl<C: Codec, L: len::Length> Clone for PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

impl<C: Codec, L: len::Length> fmt::Debug for PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PackedArray").field(&self.store).finish()
    }
}

impl<C: Codec, L: len::Length> PartialEq for PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    fn eq(&self, other: &Self) -> bool {
        self.store == other.store
    }
}

impl<C: Codec, L: len::Length> Eq for PackedArray<C, L> where C: layout::Layout<L> {}

impl<C: Codec, L: len::Length> Hash for PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.store.hash(state);
    }
}

impl<C: Codec, L: len::Length> PartialOrd for PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Codec, L: len::Length> Ord for PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.store.cmp(&other.store)
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
{
    pub const BITS: NonZeroUsize = <C as scalar::Layout>::BITS;
    pub const LEN: NonZeroUsize = L::LEN;
    pub const USED_BITS: NonZeroUsize =
        NonZeroUsize::new(Self::BITS.get() * Self::LEN.get()).unwrap();
    pub const CAPACITY_BITS: NonZeroUsize =
        <<C as layout::Layout<L>>::Store as StaticStore>::CAPACITY_BITS;
    pub const PADDING_BITS: usize = Self::CAPACITY_BITS.get() - Self::USED_BITS.get();

    /// Creates an array without checking the layout or codec invariant.
    ///
    /// # Safety
    ///
    /// The first scalar field must occupy the most significant used bits of
    /// `store`, and every unused low-order padding bit after
    /// [`Self::USED_BITS`] must be zero. Every packed field must also be valid
    /// for codec `C`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(store: <C as layout::Layout<L>>::Store) -> Self {
        Self { store }
    }

    #[inline(always)]
    pub const fn store(&self) -> &<C as layout::Layout<L>>::Store {
        &self.store
    }

    #[inline(always)]
    pub fn into_store(self) -> <C as layout::Layout<L>>::Store {
        self.store
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: ScalarGet<<C as scalar::Layout>::Store>,
{
    /// Returns the scalar at `pos`, or `None` when `pos >= L::LEN`.
    #[inline(always)]
    #[must_use]
    pub fn get(&self, pos: usize) -> Option<scalar::Scalar<C>> {
        if pos >= Self::LEN.get() {
            return None;
        }

        Some(unsafe { self.get_unchecked(pos) })
    }

    #[inline(always)]
    pub fn first(&self) -> scalar::Scalar<C> {
        let store = unsafe { self.store.first_unchecked::<C>() };

        unsafe { scalar::Scalar::new_unchecked(store) }
    }

    #[inline(always)]
    pub fn last(&self) -> scalar::Scalar<C> {
        unsafe { self.get_unchecked(Self::LEN.get() - 1) }
    }

    /// Returns the scalar at `pos` without checking whether `pos` is in bounds.
    ///
    /// # Safety
    ///
    /// `pos` must be less than `L::LEN`.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, pos: usize) -> scalar::Scalar<C> {
        let store = unsafe { self.store.get_unchecked::<C>(pos) };

        unsafe { scalar::Scalar::new_unchecked(store) }
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: ScalarSet<<C as scalar::Layout>::Store>,
{
    /// Replaces the scalar at `pos`, returning `false` when `pos >= L::LEN`.
    #[inline(always)]
    #[must_use]
    pub fn set(&mut self, pos: usize, scalar: scalar::Scalar<C>) -> bool {
        if pos >= Self::LEN.get() {
            return false;
        }

        unsafe { self.set_unchecked(pos, scalar) };

        true
    }

    /// Replaces the scalar at `pos` without checking whether `pos` is in bounds.
    ///
    /// # Safety
    ///
    /// `pos` must be less than `L::LEN`.
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, pos: usize, scalar: scalar::Scalar<C>) {
        unsafe { self.store.set_unchecked::<C>(pos, scalar.store()) };
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Pack<<C as scalar::Layout>::Store>,
{
    /// Creates an array from a fixed-length scalar array.
    #[inline(always)]
    #[must_use]
    pub fn from_array(scalars: &L::Array<scalar::Scalar<C>>) -> Self {
        let stores = scalar::Scalar::<C>::as_store_slice(scalars.as_ref());
        let store = unsafe {
            <<C as layout::Layout<L>>::Store as Pack<<C as scalar::Layout>::Store>>::pack_unchecked::<
                C,
            >(stores)
        };

        unsafe { Self::new_unchecked(store) }
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Pack<<C as scalar::Layout>::Store>,
{
    /// Creates an array from a scalar slice, returning `None` on length mismatch.
    #[inline(always)]
    #[must_use]
    pub fn try_from_slice(scalars: &[scalar::Scalar<C>]) -> Option<Self> {
        if scalars.len() != Self::LEN.get() {
            return None;
        }

        Some(unsafe { Self::from_slice_unchecked(scalars) })
    }

    /// Creates an array from a scalar slice without checking its length.
    ///
    /// # Safety
    ///
    /// `scalars.len()` must equal `L::LEN`.
    #[inline(always)]
    pub unsafe fn from_slice_unchecked(scalars: &[scalar::Scalar<C>]) -> Self {
        let stores = scalar::Scalar::<C>::as_store_slice(scalars);
        let store = unsafe {
            <<C as layout::Layout<L>>::Store as Pack<<C as scalar::Layout>::Store>>::pack_unchecked::<
                C,
            >(stores)
        };

        unsafe { Self::new_unchecked(store) }
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Unpack<<C as scalar::Layout>::Store>,
{
    /// Converts this packed array to a fixed-length scalar array.
    #[inline(always)]
    #[must_use]
    pub fn to_array(&self) -> L::Array<scalar::Scalar<C>> {
        let mut scalars = L::array_from_fn(|_| scalar::Scalar::<C>::ZERO);
        let stores = unsafe { scalar::Scalar::<C>::as_store_slice_mut_unchecked(scalars.as_mut()) };

        unsafe { self.store.unpack_unchecked::<C>(stores) };

        scalars
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Unpack<<C as scalar::Layout>::Store>,
{
    #[inline(always)]
    #[must_use]
    pub fn copy_to_slice(&self, scalars: &mut [scalar::Scalar<C>]) -> bool {
        if scalars.len() != Self::LEN.get() {
            return false;
        }

        unsafe { self.copy_to_slice_unchecked(scalars) };

        true
    }

    /// Copies scalars into a slice without checking its length.
    ///
    /// # Safety
    ///
    /// `scalars.len()` must equal `L::LEN`.
    #[inline(always)]
    pub unsafe fn copy_to_slice_unchecked(&self, scalars: &mut [scalar::Scalar<C>]) {
        let stores = unsafe { scalar::Scalar::<C>::as_store_slice_mut_unchecked(scalars) };

        unsafe { self.store.unpack_unchecked::<C>(stores) };
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>,
{
    /// Reverses scalar order in place.
    #[inline(always)]
    pub fn reverse(&mut self) {
        if Self::LEN.get() > 1 {
            unsafe {
                <<C as layout::Layout<L>>::Store as Reverse<
                    C::Bits,
                    <C::Bits as scalar::Layout>::Store,
                >>::reverse_unchecked(&mut self.store, Self::LEN.get())
            };
        }
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: BitComplement + layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Complement<<C as scalar::Layout>::Store>,
{
    /// Replaces every scalar with its semantic complement.
    #[inline(always)]
    pub fn complement(&mut self) {
        unsafe { self.store.complement_unchecked::<C>(Self::LEN.get()) };
    }
}

impl<C: Codec, L: len::Length> PackedArray<C, L>
where
    C: layout::Layout<L>,
    <C as layout::Layout<L>>::Store: Slide<<C as scalar::Layout>::Store>,
{
    /// Prepends `scalar`, shifting existing scalars right and dropping the last.
    #[inline(always)]
    pub fn slide_left(&mut self, scalar: scalar::Scalar<C>) {
        unsafe { self.store.slide_left_unchecked::<C, L>(scalar.store()) };
    }

    /// Appends `scalar`, shifting existing scalars left and dropping the first.
    #[inline(always)]
    pub fn slide_right(&mut self, scalar: scalar::Scalar<C>) {
        unsafe { self.store.slide_right_unchecked::<C, L>(scalar.store()) };
    }
}

impl<C: Codec, L: len::Next> PackedArray<C, L>
where
    C: layout::Layout<L> + layout::Layout<<L as len::Next>::Next>,
    <C as layout::Layout<L>>::Store: GrowLeft<<C as layout::Layout<<L as len::Next>::Next>>::Store, <C as scalar::Layout>::Store>,
{
    /// Prepends `scalar`, returning the successor-length array.
    #[inline(always)]
    pub fn push_left(self, scalar: scalar::Scalar<C>) -> PackedArray<C, <L as len::Next>::Next> {
        let store = unsafe { self.store.grow_left_unchecked::<C, L>(scalar.store()) };

        unsafe { PackedArray::<C, <L as len::Next>::Next>::new_unchecked(store) }
    }
}

impl<C: Codec, L: len::Next> PackedArray<C, L>
where
    C: layout::Layout<L> + layout::Layout<<L as len::Next>::Next>,
    <C as layout::Layout<L>>::Store: GrowRight<
            <C as layout::Layout<<L as len::Next>::Next>>::Store,
            <C as scalar::Layout>::Store,
        >,
{
    /// Appends `scalar`, returning the successor-length array.
    #[inline(always)]
    pub fn push_right(self, scalar: scalar::Scalar<C>) -> PackedArray<C, <L as len::Next>::Next> {
        let store = unsafe { self.store.grow_right_unchecked::<C, L>(scalar.store()) };

        unsafe { PackedArray::<C, <L as len::Next>::Next>::new_unchecked(store) }
    }
}

impl<C: Codec, L: len::Prev> PackedArray<C, L>
where
    C: layout::Layout<L> + layout::Layout<<L as len::Prev>::Prev>,
    <C as layout::Layout<L>>::Store: ShrinkLeft<
            <C as layout::Layout<<L as len::Prev>::Prev>>::Store,
            <C as scalar::Layout>::Store,
        >,
{
    /// Drops the first scalar, returning the predecessor-length array.
    #[inline(always)]
    pub fn drop_left(self) -> PackedArray<C, <L as len::Prev>::Prev> {
        let store = unsafe { self.store.shrink_left_unchecked::<C, L>() };

        unsafe { PackedArray::<C, <L as len::Prev>::Prev>::new_unchecked(store) }
    }
}

impl<C: Codec, L: len::Prev> PackedArray<C, L>
where
    C: layout::Layout<L> + layout::Layout<<L as len::Prev>::Prev>,
    <C as layout::Layout<L>>::Store: ShrinkRight<
            <C as layout::Layout<<L as len::Prev>::Prev>>::Store,
            <C as scalar::Layout>::Store,
        >,
{
    /// Drops the last scalar, returning the predecessor-length array.
    #[inline(always)]
    pub fn drop_right(self) -> PackedArray<C, <L as len::Prev>::Prev> {
        let store = unsafe { self.store.shrink_right_unchecked::<C, L>() };

        unsafe { PackedArray::<C, <L as len::Prev>::Prev>::new_unchecked(store) }
    }
}
