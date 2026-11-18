use crate::{BitComplement, Codec, len, scalar, store};
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use store::{
    Complement, DropLeft, DropRight, PushLeft, PushRight, Reverse, ScalarSet, Slide, Unpack,
};

/// Growable packed scalar vector.
///
/// `PackedVec<C>` stores codec-valid scalars in a growable `Vec<u64>`.
/// Active scalars occupy the most significant bits of the initialized words,
/// and unused low-order bits in the final word are zero.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackedVec<C>
where
    C: Codec,
{
    words: Vec<u64>,
    len: usize,
    _codec: PhantomData<C>,
}

impl<C> PackedVec<C>
where
    C: Codec,
{
    pub const BITS: NonZeroUsize = <C as scalar::Layout>::BITS;
    pub const WORD_BITS: usize = u64::BITS as usize;

    /// Creates an empty vector.
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            words: Vec::new(),
            len: 0,
            _codec: PhantomData,
        }
    }

    /// Creates an empty vector with capacity for at least `capacity` scalars.
    #[inline(always)]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            words: Vec::with_capacity(Self::words_for_len(capacity)),
            len: 0,
            _codec: PhantomData,
        }
    }

    /// Creates a vector without checking length, padding, or codec invariants.
    ///
    /// # Safety
    ///
    /// `words.len()` must equal the number of words required for `len`
    /// scalars. The first `len` scalar fields must be valid for codec `C`,
    /// and every unused low-order bit in the final word must be zero.
    #[inline(always)]
    #[must_use]
    pub unsafe fn new_unchecked(words: Vec<u64>, len: usize) -> Self {
        Self {
            words,
            len,
            _codec: PhantomData,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    #[inline(always)]
    #[must_use]
    pub fn into_words(self) -> Vec<u64> {
        self.words
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
    pub fn capacity(&self) -> usize {
        self.capacity_bits() / Self::BITS
    }

    #[inline(always)]
    #[must_use]
    pub fn remaining_capacity(&self) -> usize {
        self.capacity().saturating_sub(self.len)
    }

    #[inline(always)]
    #[must_use]
    pub const fn word_len(&self) -> usize {
        self.words.len()
    }

    #[inline(always)]
    #[must_use]
    pub fn word_capacity(&self) -> usize {
        self.words.capacity()
    }

    #[inline(always)]
    #[must_use]
    pub const fn used_bits(&self) -> usize {
        Self::BITS.get() * self.len
    }

    #[inline(always)]
    #[must_use]
    pub fn capacity_bits(&self) -> usize {
        store::Store::capacity_bits(&self.words)
    }

    #[inline(always)]
    #[must_use]
    pub const fn padding_bits(&self) -> usize {
        (self.words.len() * Self::WORD_BITS) - self.used_bits()
    }

    #[inline(always)]
    fn words_for_len(len: usize) -> usize {
        len.checked_mul(Self::BITS.get())
            .expect("packed vector bit capacity overflow")
            .div_ceil(Self::WORD_BITS)
    }

    #[inline(always)]
    fn reserve_words(&mut self, len: usize) {
        let words = Self::words_for_len(len);

        if words > self.words.capacity() {
            self.words.reserve(words - self.words.capacity());
        }
    }

    /// Reserves capacity for at least `additional` more scalars.
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        let len = self
            .len
            .checked_add(additional)
            .expect("packed vector length overflow");

        self.reserve_words(len);
    }

    /// Removes all scalars.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.words.clear();
        self.len = 0;
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    [u64]: store::ScalarGet<<C as scalar::Layout>::Store>,
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
        let store = unsafe {
            <[u64] as store::ScalarGet<<C as scalar::Layout>::Store>>::get_unchecked::<C>(
                self.words.as_slice(),
                pos,
            )
        };

        unsafe { scalar::Scalar::new_unchecked(store) }
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: ScalarSet<<C as scalar::Layout>::Store>,
{
    /// Creates a vector from a fixed-length scalar array.
    #[inline(always)]
    #[must_use]
    pub fn from_array<L>(scalars: &L::Array<scalar::Scalar<C>>) -> Self
    where
        L: len::Length,
    {
        unsafe { Self::from_slice_unchecked(scalars.as_ref()) }
    }

    /// Creates a vector from a scalar slice.
    #[inline(always)]
    #[must_use]
    pub fn from_slice(scalars: &[scalar::Scalar<C>]) -> Self {
        unsafe { Self::from_slice_unchecked(scalars) }
    }

    /// Creates a vector from a scalar slice without checking scalar invariants.
    ///
    /// # Safety
    ///
    /// Every scalar must be valid for codec `C`.
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_slice_unchecked(scalars: &[scalar::Scalar<C>]) -> Self {
        let len = scalars.len();
        let mut vector = Self {
            words: vec![0; Self::words_for_len(len)],
            len,
            _codec: PhantomData,
        };

        for (pos, scalar) in scalars.iter().copied().enumerate() {
            unsafe { vector.words.set_unchecked::<C>(pos, scalar.store()) };
        }

        vector
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: Unpack<<C as scalar::Layout>::Store>,
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
        let mut scalars = L::array_from_fn(|_| unsafe {
            scalar::Scalar::new_unchecked(<<C as scalar::Layout>::Store as scalar::Store>::ZERO)
        });
        let stores = unsafe { scalar::Scalar::<C>::as_store_slice_mut_unchecked(scalars.as_mut()) };

        unsafe { self.words.unpack_unchecked::<C>(stores) };

        scalars
    }

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

        unsafe { self.words.unpack_unchecked::<C>(stores) };
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: ScalarSet<<C as scalar::Layout>::Store>,
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
        unsafe { self.words.set_unchecked::<C>(pos, scalar.store()) };
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>,
{
    /// Reverses scalar order in place.
    #[inline(always)]
    pub fn reverse(&mut self) {
        if self.len > 1 {
            unsafe {
                <Vec<u64> as Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>>::reverse_unchecked(
                    &mut self.words,
                    self.len,
                )
            };
        }
    }
}

impl<C> PackedVec<C>
where
    C: BitComplement,
    Vec<u64>: Complement<<C as scalar::Layout>::Store>,
{
    /// Replaces every active scalar with its semantic complement.
    #[inline(always)]
    pub fn complement(&mut self) {
        if self.len > 0 {
            unsafe { self.words.complement_unchecked::<C>(self.len) };
        }
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: Slide<<C as scalar::Layout>::Store>,
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
            self.words
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
            self.words
                .slide_right_runtime_unchecked::<C>(self.len, scalar.store())
        };
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: PushLeft<<C as scalar::Layout>::Store>,
{
    /// Prepends `scalar`.
    #[inline(always)]
    pub fn push_left(&mut self, scalar: scalar::Scalar<C>) {
        unsafe { self.push_left_unchecked(scalar) };
    }

    /// Prepends `scalar` without checking scalar invariants.
    ///
    /// # Safety
    ///
    /// `scalar` must be valid for codec `C`.
    #[inline(always)]
    pub unsafe fn push_left_unchecked(&mut self, scalar: scalar::Scalar<C>) {
        let old_len = self.len;
        let new_len = old_len
            .checked_add(1)
            .expect("packed vector length overflow");

        self.words.resize(Self::words_for_len(new_len), 0);
        unsafe { self.words.push_left_unchecked::<C>(old_len, scalar.store()) };
        self.len = new_len;
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: PushRight<<C as scalar::Layout>::Store>,
{
    /// Appends `scalar`.
    #[inline(always)]
    pub fn push_right(&mut self, scalar: scalar::Scalar<C>) {
        unsafe { self.push_right_unchecked(scalar) };
    }

    /// Appends `scalar` without checking scalar invariants.
    ///
    /// # Safety
    ///
    /// `scalar` must be valid for codec `C`.
    #[inline(always)]
    pub unsafe fn push_right_unchecked(&mut self, scalar: scalar::Scalar<C>) {
        let old_len = self.len;
        let new_len = old_len
            .checked_add(1)
            .expect("packed vector length overflow");

        self.words.resize(Self::words_for_len(new_len), 0);
        unsafe {
            self.words
                .push_right_unchecked::<C>(old_len, scalar.store())
        };
        self.len = new_len;
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: DropLeft<<C as scalar::Layout>::Store>,
{
    /// Removes the first scalar, returning `false` when the vector is empty.
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
        unsafe { self.words.drop_left_unchecked::<C>(self.len) };
        self.len -= 1;
        self.words.truncate(Self::words_for_len(self.len));
    }
}

impl<C> PackedVec<C>
where
    C: Codec,
    Vec<u64>: DropRight<<C as scalar::Layout>::Store>,
{
    /// Removes the last scalar, returning `false` when the vector is empty.
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
        unsafe { self.words.drop_right_unchecked::<C>(self.len) };
        self.len -= 1;
        self.words.truncate(Self::words_for_len(self.len));
    }
}

impl<C> Default for PackedVec<C>
where
    C: Codec,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
