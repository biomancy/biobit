use crate::{BitComplement, Codec, len, scalar, store};
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use store::{
    Complement, DropLeft, DropRight, Pack, PushLeft, PushRight, Reverse, ScalarGet, ScalarSet,
    Slide, StaticStore, Unpack,
};

/// Fixed-capacity packed scalar buffer.
///
/// `PackedBuffer<C, S>` stores up to `CAPACITY` scalars for codec `C` in the
/// abstract numeric value of store `S`. Active scalars occupy the most
/// significant slots, and unused low-order bits are zero.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore,
{
    store: S,
    len: usize,
    _codec: PhantomData<C>,
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore,
{
    pub const BITS: NonZeroUsize = <C as scalar::Layout>::BITS;
    pub const CAPACITY_BITS: NonZeroUsize = S::CAPACITY_BITS;
    pub const CAPACITY: usize = Self::CAPACITY_BITS.get() / Self::BITS.get();

    /// Creates an empty buffer.
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            store: S::ZERO,
            len: 0,
            _codec: PhantomData,
        }
    }

    /// Creates a buffer without checking length, padding, or codec invariants.
    ///
    /// # Safety
    ///
    /// `len` must not exceed `CAPACITY`. The first `len` scalar fields must be
    /// valid for codec `C`, and every unused low-order bit must be zero.
    #[inline(always)]
    #[must_use]
    pub const unsafe fn new_unchecked(store: S, len: usize) -> Self {
        Self {
            store,
            len,
            _codec: PhantomData,
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn store(&self) -> &S {
        &self.store
    }

    #[inline(always)]
    #[must_use]
    pub fn into_store(self) -> S {
        self.store
    }

    #[inline(always)]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len == Self::CAPACITY
    }

    #[inline(always)]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    #[inline(always)]
    #[must_use]
    pub const fn remaining_capacity(&self) -> usize {
        Self::CAPACITY - self.len
    }

    #[inline(always)]
    #[must_use]
    pub const fn used_bits(&self) -> usize {
        Self::BITS.get() * self.len
    }

    #[inline(always)]
    #[must_use]
    pub const fn padding_bits(&self) -> usize {
        Self::CAPACITY_BITS.get() - self.used_bits()
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + ScalarGet<<C as scalar::Layout>::Store>,
{
    /// Returns the scalar at `pos`, or `None` when `pos >= len`.
    #[inline(always)]
    #[must_use]
    pub fn get(&self, pos: usize) -> Option<scalar::Scalar<C>> {
        if pos >= self.len {
            return None;
        }

        Some(unsafe { self.get_unchecked(pos) })
    }

    /// Returns the scalar at `pos` without checking whether `pos` is in bounds.
    ///
    /// # Safety
    ///
    /// `pos` must be less than `self.len()`.
    #[inline(always)]
    #[must_use]
    pub unsafe fn get_unchecked(&self, pos: usize) -> scalar::Scalar<C> {
        let store = unsafe { self.store.get_unchecked::<C>(pos) };

        unsafe { scalar::Scalar::new_unchecked(store) }
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + Pack<<C as scalar::Layout>::Store>,
{
    /// Creates a buffer from a fixed-length scalar array.
    ///
    /// Returns `None` when the array exceeds capacity.
    #[inline(always)]
    #[must_use]
    pub fn try_from_array<L>(scalars: &L::Array<scalar::Scalar<C>>) -> Option<Self>
    where
        L: len::Length,
    {
        if L::LEN.get() > Self::CAPACITY {
            return None;
        }

        Some(unsafe { Self::from_array_unchecked::<L>(scalars) })
    }

    /// Creates a buffer from a fixed-length scalar array without checking capacity.
    ///
    /// # Safety
    ///
    /// `L::LEN` must not exceed `Self::CAPACITY`.
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_array_unchecked<L>(scalars: &L::Array<scalar::Scalar<C>>) -> Self
    where
        L: len::Length,
    {
        let stores = scalar::Scalar::<C>::as_store_slice(scalars.as_ref());
        let store =
            unsafe { <S as Pack<<C as scalar::Layout>::Store>>::pack_unchecked::<C>(stores) };

        unsafe { Self::new_unchecked(store, L::LEN.get()) }
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + Pack<<C as scalar::Layout>::Store>,
{
    /// Creates a buffer from a scalar slice.
    ///
    /// Returns `None` when the slice exceeds capacity.
    #[inline(always)]
    #[must_use]
    pub fn try_from_slice(scalars: &[scalar::Scalar<C>]) -> Option<Self> {
        if scalars.len() > Self::CAPACITY {
            return None;
        }

        Some(unsafe { Self::from_slice_unchecked(scalars) })
    }

    /// Creates a buffer from a scalar slice without checking capacity.
    ///
    /// # Safety
    ///
    /// `scalars.len()` must not exceed `Self::CAPACITY`.
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_slice_unchecked(scalars: &[scalar::Scalar<C>]) -> Self {
        let stores = scalar::Scalar::<C>::as_store_slice(scalars);
        let store =
            unsafe { <S as Pack<<C as scalar::Layout>::Store>>::pack_unchecked::<C>(stores) };

        unsafe { Self::new_unchecked(store, scalars.len()) }
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + Unpack<<C as scalar::Layout>::Store>,
{
    /// Converts to a fixed-length scalar array, returning `None` on length mismatch.
    #[inline(always)]
    #[must_use]
    pub fn try_to_array<L>(&self) -> Option<L::Array<scalar::Scalar<C>>>
    where
        L: len::Length,
    {
        if L::LEN.get() != self.len {
            return None;
        }

        Some(unsafe { self.to_array_unchecked::<L>() })
    }

    /// Converts to a fixed-length scalar array without checking length.
    ///
    /// # Safety
    ///
    /// `L::LEN` must equal `self.len()`.
    #[inline(always)]
    #[must_use]
    pub unsafe fn to_array_unchecked<L>(&self) -> L::Array<scalar::Scalar<C>>
    where
        L: len::Length,
    {
        let mut scalars = L::array_from_fn(|_| scalar::Scalar::<C>::ZERO);
        let stores = unsafe { scalar::Scalar::<C>::as_store_slice_mut_unchecked(scalars.as_mut()) };

        unsafe { self.store.unpack_unchecked::<C>(stores) };

        scalars
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + Unpack<<C as scalar::Layout>::Store>,
{
    /// Copies scalars to a slice, returning `false` on length mismatch.
    #[inline(always)]
    #[must_use]
    pub fn copy_to_slice(&self, scalars: &mut [scalar::Scalar<C>]) -> bool {
        if scalars.len() != self.len {
            return false;
        }

        unsafe { self.copy_to_slice_unchecked(scalars) };

        true
    }

    /// Copies scalars to a slice without checking length.
    ///
    /// # Safety
    ///
    /// `scalars.len()` must equal `self.len()`.
    #[inline(always)]
    pub unsafe fn copy_to_slice_unchecked(&self, scalars: &mut [scalar::Scalar<C>]) {
        let stores = unsafe { scalar::Scalar::<C>::as_store_slice_mut_unchecked(scalars) };

        unsafe { self.store.unpack_unchecked::<C>(stores) };
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>,
{
    /// Reverses scalar order in place.
    #[inline(always)]
    pub fn reverse(&mut self) {
        if self.len > 1 {
            unsafe {
                <S as Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>>::reverse_unchecked(
                    &mut self.store,
                    self.len,
                )
            };
        }
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: BitComplement,
    S: StaticStore + Complement<<C as scalar::Layout>::Store>,
{
    /// Replaces every active scalar with its semantic complement.
    #[inline(always)]
    pub fn complement(&mut self) {
        if self.len > 0 {
            unsafe { self.store.complement_unchecked::<C>(self.len) };
        }
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + Slide<<C as scalar::Layout>::Store>,
{
    /// Prepends `scalar`, shifting existing scalars right and dropping the last.
    #[inline(always)]
    #[must_use]
    pub fn slide_left(&mut self, scalar: scalar::Scalar<C>) -> bool {
        if self.is_empty() {
            return false;
        }

        unsafe { self.slide_left_unchecked(scalar) };

        true
    }

    /// Prepends `scalar` without checking emptiness.
    ///
    /// # Safety
    ///
    /// `self.len()` must be greater than zero.
    #[inline(always)]
    pub unsafe fn slide_left_unchecked(&mut self, scalar: scalar::Scalar<C>) {
        unsafe {
            self.store
                .slide_left_runtime_unchecked::<C>(self.len, scalar.store())
        };
    }

    /// Appends `scalar`, shifting existing scalars left and dropping the first.
    #[inline(always)]
    #[must_use]
    pub fn slide_right(&mut self, scalar: scalar::Scalar<C>) -> bool {
        if self.is_empty() {
            return false;
        }

        unsafe { self.slide_right_unchecked(scalar) };

        true
    }

    /// Appends `scalar` without checking emptiness.
    ///
    /// # Safety
    ///
    /// `self.len()` must be greater than zero.
    #[inline(always)]
    pub unsafe fn slide_right_unchecked(&mut self, scalar: scalar::Scalar<C>) {
        unsafe {
            self.store
                .slide_right_runtime_unchecked::<C>(self.len, scalar.store())
        };
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + PushLeft<<C as scalar::Layout>::Store>,
{
    /// Prepends `scalar`, returning `false` when the buffer is full.
    #[inline(always)]
    #[must_use]
    pub fn push_left(&mut self, scalar: scalar::Scalar<C>) -> bool {
        if self.is_full() {
            return false;
        }

        unsafe { self.push_left_unchecked(scalar) };

        true
    }

    /// Prepends `scalar` without checking capacity.
    ///
    /// # Safety
    ///
    /// `self.len() + 1` must not exceed `self.capacity()`.
    #[inline(always)]
    pub unsafe fn push_left_unchecked(&mut self, scalar: scalar::Scalar<C>) {
        unsafe {
            self.store
                .push_left_unchecked::<C>(self.len, scalar.store())
        };
        self.len += 1;
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + PushRight<<C as scalar::Layout>::Store>,
{
    /// Appends `scalar`, returning `false` when the buffer is full.
    #[inline(always)]
    #[must_use]
    pub fn push_right(&mut self, scalar: scalar::Scalar<C>) -> bool {
        if self.is_full() {
            return false;
        }

        unsafe { self.push_right_unchecked(scalar) };

        true
    }

    /// Appends `scalar` without checking capacity.
    ///
    /// # Safety
    ///
    /// `self.len() + 1` must not exceed `self.capacity()`.
    #[inline(always)]
    pub unsafe fn push_right_unchecked(&mut self, scalar: scalar::Scalar<C>) {
        unsafe {
            self.store
                .push_right_unchecked::<C>(self.len, scalar.store())
        };
        self.len += 1;
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + DropLeft<<C as scalar::Layout>::Store>,
{
    /// Removes the first scalar, returning `false` when the buffer is empty.
    #[inline(always)]
    #[must_use]
    pub fn drop_left(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        unsafe { self.drop_left_unchecked() };

        true
    }

    /// Removes the first scalar without checking emptiness.
    ///
    /// # Safety
    ///
    /// `self.len()` must be greater than zero.
    #[inline(always)]
    pub unsafe fn drop_left_unchecked(&mut self) {
        unsafe { self.store.drop_left_unchecked::<C>(self.len) };
        self.len -= 1;
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + DropRight<<C as scalar::Layout>::Store>,
{
    /// Removes the last scalar, returning `false` when the buffer is empty.
    #[inline(always)]
    #[must_use]
    pub fn drop_right(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        unsafe { self.drop_right_unchecked() };

        true
    }

    /// Removes the last scalar without checking emptiness.
    ///
    /// # Safety
    ///
    /// `self.len()` must be greater than zero.
    #[inline(always)]
    pub unsafe fn drop_right_unchecked(&mut self) {
        unsafe { self.store.drop_right_unchecked::<C>(self.len) };
        self.len -= 1;
    }
}

impl<C, S> PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore + ScalarSet<<C as scalar::Layout>::Store>,
{
    /// Replaces the scalar at `pos`, returning `false` when `pos >= len`.
    #[inline(always)]
    #[must_use]
    pub fn set(&mut self, pos: usize, scalar: scalar::Scalar<C>) -> bool {
        if pos >= self.len {
            return false;
        }

        unsafe { self.set_unchecked(pos, scalar) };

        true
    }

    /// Replaces the scalar at `pos` without checking whether `pos` is in bounds.
    ///
    /// # Safety
    ///
    /// `pos` must be less than `self.len()`.
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, pos: usize, scalar: scalar::Scalar<C>) {
        unsafe { self.store.set_unchecked::<C>(pos, scalar.store()) };
    }
}

impl<C, S> Default for PackedBuffer<C, S>
where
    C: Codec,
    S: StaticStore,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
