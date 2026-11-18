use crate::{len, scalar};
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;

/// Physical storage for packed scalar fields.
///
/// The canonical invariant is enforced by the frontend containers: in the
/// abstract numeric value, the first scalar occupies the most significant used
/// bits, subsequent scalars move toward less significant bits, and every
/// unused low-order padding bit is zero.
pub trait Store: Debug + Send + Sync {
    /// Returns the number of bits addressable by this store value.
    #[must_use]
    fn capacity_bits(&self) -> usize;
}

/// Fixed-capacity owned storage for packed scalar fields.
pub trait StaticStore:
    Store + Clone + Eq + Ord + Debug + Hash + Send + Sync + Sized + 'static
{
    /// Zero-filled canonical store value.
    const ZERO: Self;

    /// Compile-time bit capacity for this store type.
    const CAPACITY_BITS: NonZeroUsize;
}

/// Flips every bit in the active scalar fields of a packed store.
///
/// `ScalarStore` is the low-bit aligned scalar result type. `ScalarLayout`
/// supplies the field width for each method call.
///
/// # Safety
///
/// Position `0` must be the most significant packed field.
#[doc(hidden)]
pub unsafe trait Complement<ScalarStore: scalar::Store>: Store {
    /// Complements active scalar fields without checking length or layout.
    ///
    /// # Safety
    ///
    /// `len` must be greater than zero, all `len` scalar fields must fit in
    /// `Self`, and unused low-order padding bits must be zero.
    unsafe fn complement_unchecked<ScalarLayout>(&mut self, len: usize)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Removes the first scalar field in place without changing the store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait DropLeft<ScalarStore: scalar::Store>: Store {
    /// Drops the first scalar: `(A, B, C)` becomes `(B, C)`.
    ///
    /// # Safety
    ///
    /// `len` must be the current active length and greater than zero, all
    /// remaining fields must fit in `Self`, and `self` must have canonical
    /// padding for `len`.
    unsafe fn drop_left_unchecked<ScalarLayout>(&mut self, len: usize)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Removes the last scalar field in place without changing the store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait DropRight<ScalarStore: scalar::Store>: Store {
    /// Drops the last scalar: `(A, B, C)` becomes `(A, B)`.
    ///
    /// # Safety
    ///
    /// `len` must be the current active length and greater than zero, all
    /// remaining fields must fit in `Self`, and `self` must have canonical
    /// padding for `len`.
    unsafe fn drop_right_unchecked<ScalarLayout>(&mut self, len: usize)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Reads scalar fields from a packed store.
///
/// `ScalarStore` is the low-bit aligned result type. `ScalarLayout` supplies
/// the field width and mask for each method call.
///
/// # Safety
///
/// Position `0` must be the most significant packed field.
#[doc(hidden)]
pub unsafe trait ScalarGet<ScalarStore: scalar::Store>: Store {
    /// Returns the first scalar without checking that it fits in `Self`.
    ///
    /// # Safety
    ///
    /// The first scalar field must fit in `Self`.
    unsafe fn first_unchecked<ScalarLayout>(&self) -> ScalarStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;

    /// Returns the scalar at `pos` without checking that it fits in `Self`.
    ///
    /// # Safety
    ///
    /// The scalar field at `pos` must fit in `Self`.
    unsafe fn get_unchecked<ScalarLayout>(&self, pos: usize) -> ScalarStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Packs scalar stores into a packed store.
///
/// `ScalarStore` is the low-bit aligned input type. `ScalarLayout` supplies the
/// field width and mask for each method call.
///
/// # Safety
///
/// The first scalar must occupy the most significant used field.
#[doc(hidden)]
pub unsafe trait Pack<ScalarStore: scalar::Store>: StaticStore {
    /// Packs every low-bit aligned scalar store into a new backing store.
    ///
    /// # Safety
    ///
    /// All scalar fields must fit in `Self`, and every scalar must not
    /// contain any set bit outside `ScalarLayout`'s scalar mask.
    unsafe fn pack_unchecked<ScalarLayout>(scalars: &[ScalarStore]) -> Self
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Prepends one scalar field in place without changing the store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait PushLeft<ScalarStore: scalar::Store>: Store {
    /// Prepends `scalar`: `(A, B, C)` becomes `(scalar, A, B, C)`.
    ///
    /// # Safety
    ///
    /// `len` must be the current active length, `len + 1` fields must fit in
    /// `Self`, `self` must have canonical padding for `len`, and `scalar`
    /// must fit in `ScalarLayout`.
    unsafe fn push_left_unchecked<ScalarLayout>(&mut self, len: usize, scalar: ScalarStore)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Appends one scalar field in place without changing the store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait PushRight<ScalarStore: scalar::Store>: Store {
    /// Appends `scalar`: `(A, B, C)` becomes `(A, B, C, scalar)`.
    ///
    /// # Safety
    ///
    /// `len` must be the current active length, `len + 1` fields must fit in
    /// `Self`, `self` must have canonical padding for `len`, and `scalar`
    /// must fit in `ScalarLayout`.
    unsafe fn push_right_unchecked<ScalarLayout>(&mut self, len: usize, scalar: ScalarStore)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Removes the first scalar field into a narrower packed store.
///
/// `PrevStore` is the physical store for the predecessor length.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait NarrowLeft<PrevStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Drops the first scalar: `(A, B, C)` becomes `(B, C)`.
    ///
    /// # Safety
    ///
    /// All current and previous fields must fit in their stores, and `self`
    /// must have canonical padding.
    unsafe fn narrow_left_unchecked<ScalarLayout, L>(self) -> PrevStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Prev;
}

/// Removes the last scalar field into a narrower packed store.
///
/// `PrevStore` is the physical store for the predecessor length.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait NarrowRight<PrevStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Drops the last scalar: `(A, B, C)` becomes `(A, B)`.
    ///
    /// # Safety
    ///
    /// All current and previous fields must fit in their stores, and `self`
    /// must have canonical padding.
    unsafe fn narrow_right_unchecked<ScalarLayout, L>(self) -> PrevStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Prev;
}

/// Prepends one scalar field into a wider packed store.
///
/// `NextStore` is the physical store for the successor length.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait WidenLeft<NextStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Prepends `scalar`: `(A, B, C)` becomes `(scalar, A, B, C)`.
    ///
    /// # Safety
    ///
    /// All current and next fields must fit in their stores, `self` must have
    /// canonical padding, and `scalar` must fit in `ScalarLayout`.
    unsafe fn widen_left_unchecked<ScalarLayout, L>(self, scalar: ScalarStore) -> NextStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Next;
}

/// Appends one scalar field into a wider packed store.
///
/// `NextStore` is the physical store for the successor length.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait WidenRight<NextStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Appends `scalar`: `(A, B, C)` becomes `(A, B, C, scalar)`.
    ///
    /// # Safety
    ///
    /// All current and next fields must fit in their stores, `self` must have
    /// canonical padding, and `scalar` must fit in `ScalarLayout`.
    unsafe fn widen_right_unchecked<ScalarLayout, L>(self, scalar: ScalarStore) -> NextStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Next;
}

/// Prepends one scalar field into the successor-length store.
///
/// This fixed-array transition dispatches to [`PushLeft`] when the successor
/// layout uses the same store type, or to [`WidenLeft`] when the successor
/// layout needs a wider store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait GrowLeft<NextStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Prepends `scalar`: `(A, B, C)` becomes `(scalar, A, B, C)`.
    ///
    /// # Safety
    ///
    /// `L::LEN` must be the current active length, the successor length must
    /// fit in `NextStore`, `self` must have canonical padding for `L::LEN`,
    /// and `scalar` must fit in `ScalarLayout`.
    unsafe fn grow_left_unchecked<ScalarLayout, L>(self, scalar: ScalarStore) -> NextStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Next;
}

/// Appends one scalar field into the successor-length store.
///
/// This fixed-array transition dispatches to [`PushRight`] when the successor
/// layout uses the same store type, or to [`WidenRight`] when the successor
/// layout needs a wider store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait GrowRight<NextStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Appends `scalar`: `(A, B, C)` becomes `(A, B, C, scalar)`.
    ///
    /// # Safety
    ///
    /// `L::LEN` must be the current active length, the successor length must
    /// fit in `NextStore`, `self` must have canonical padding for `L::LEN`,
    /// and `scalar` must fit in `ScalarLayout`.
    unsafe fn grow_right_unchecked<ScalarLayout, L>(self, scalar: ScalarStore) -> NextStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Next;
}

/// Removes the first scalar field into the predecessor-length store.
///
/// This fixed-array transition dispatches to [`DropLeft`] when the predecessor
/// layout uses the same store type, or to [`NarrowLeft`] when the predecessor
/// layout needs a narrower store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait ShrinkLeft<PrevStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Drops the first scalar: `(A, B, C)` becomes `(B, C)`.
    ///
    /// # Safety
    ///
    /// `L::LEN` must be the current active length and greater than zero, the
    /// predecessor length must fit in `PrevStore`, and `self` must have
    /// canonical padding for `L::LEN`.
    unsafe fn shrink_left_unchecked<ScalarLayout, L>(self) -> PrevStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Prev;
}

/// Removes the last scalar field into the predecessor-length store.
///
/// This fixed-array transition dispatches to [`DropRight`] when the predecessor
/// layout uses the same store type, or to [`NarrowRight`] when the predecessor
/// layout needs a narrower store type.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait ShrinkRight<PrevStore: StaticStore, ScalarStore: scalar::Store>:
    StaticStore
{
    /// Drops the last scalar: `(A, B, C)` becomes `(A, B)`.
    ///
    /// # Safety
    ///
    /// `L::LEN` must be the current active length and greater than zero, the
    /// predecessor length must fit in `PrevStore`, and `self` must have
    /// canonical padding for `L::LEN`.
    unsafe fn shrink_right_unchecked<ScalarLayout, L>(self) -> PrevStore
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Prev;
}

/// Reverses active scalar field order in place.
///
/// `ScalarLayout` is a trait parameter so implementations can specialize for
/// each store and scalar width pair.
///
/// # Safety
///
/// Position `0` must be the most significant packed field.
#[doc(hidden)]
pub unsafe trait Reverse<ScalarLayout, ScalarStore>: Store
where
    ScalarLayout: scalar::Layout<Store = ScalarStore>,
    ScalarStore: scalar::Store,
{
    /// Reverses active scalar fields without checking length or layout.
    ///
    /// # Safety
    ///
    /// `len` must be greater than one, all `len` scalar fields must fit in
    /// `Self`, and unused low-order padding bits must be zero.
    unsafe fn reverse_unchecked(&mut self, len: usize);
}

/// Replaces scalar fields in a packed store.
///
/// `ScalarStore` is the low-bit aligned input type. `ScalarLayout` supplies the
/// field width and mask for each method call.
///
/// # Safety
///
/// Position `0` must be the most significant packed field.
#[doc(hidden)]
pub unsafe trait ScalarSet<ScalarStore: scalar::Store>: Store {
    /// Replaces the scalar at `pos` without checking that it fits in `Self`.
    ///
    /// # Safety
    ///
    /// The scalar field at `pos` must fit in `Self`, and `scalar` must not
    /// contain any set bit outside `ScalarLayout`'s scalar mask.
    unsafe fn set_unchecked<ScalarLayout>(&mut self, pos: usize, scalar: ScalarStore)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Slides a fixed-length packed scalar window.
///
/// `ScalarStore` is the low-bit aligned inserted scalar type. `ScalarLayout`
/// supplies the field width and mask for each method call.
///
/// # Safety
///
/// Implementations must preserve the canonical packed-array layout.
#[doc(hidden)]
pub unsafe trait Slide<ScalarStore: scalar::Store>: Store {
    /// Prepends `scalar`: `(A, B, C)` becomes `(scalar, A, B)`.
    ///
    /// # Safety
    ///
    /// All fields must fit in `Self`, and `scalar` must fit in `ScalarLayout`.
    unsafe fn slide_left_unchecked<ScalarLayout, L>(&mut self, scalar: ScalarStore)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Length;

    /// Prepends `scalar` for a runtime-sized active window.
    ///
    /// # Safety
    ///
    /// `len` must be greater than zero, all fields must fit in `Self`, and
    /// `scalar` must fit in `ScalarLayout`.
    unsafe fn slide_left_runtime_unchecked<ScalarLayout>(
        &mut self,
        len: usize,
        scalar: ScalarStore,
    ) where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;

    /// Appends `scalar`: `(A, B, C)` becomes `(B, C, scalar)`.
    ///
    /// # Safety
    ///
    /// All fields must fit in `Self`, and `scalar` must fit in `ScalarLayout`.
    unsafe fn slide_right_unchecked<ScalarLayout, L>(&mut self, scalar: ScalarStore)
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>,
        L: len::Length;

    /// Appends `scalar` for a runtime-sized active window.
    ///
    /// # Safety
    ///
    /// `len` must be greater than zero, all fields must fit in `Self`, and
    /// `scalar` must fit in `ScalarLayout`.
    unsafe fn slide_right_runtime_unchecked<ScalarLayout>(
        &mut self,
        len: usize,
        scalar: ScalarStore,
    ) where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}

/// Unpacks scalar fields from a packed store.
///
/// `ScalarStore` is the low-bit aligned result type. `ScalarLayout` supplies
/// the field width and mask for each method call.
///
/// # Safety
///
/// Position `0` must be the most significant packed field.
#[doc(hidden)]
pub unsafe trait Unpack<ScalarStore: scalar::Store>: Store {
    /// Extracts scalar fields into low-bit aligned scalar stores.
    ///
    /// # Safety
    ///
    /// All output scalar fields must fit in `Self`.
    unsafe fn unpack_unchecked<ScalarLayout>(&self, scalars: &mut [ScalarStore])
    where
        ScalarLayout: scalar::Layout<Store = ScalarStore>;
}
