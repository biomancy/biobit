//! Type-erased traits used by the broker.

use crate::access::private;
use crate::access::{Result, StoreId};
use std::sync::Arc;

/// Describes an access handle to the broker without exposing its value's type.
/// The broker uses [`DynAccess::kind`] to find the stores needed for acquisition.
///
/// Handles are [`Send`] so ownership can move between threads.
pub trait DynAccess: private::Sealed + Send {
    /// Describes how the broker should acquire this handle.
    fn kind(&self) -> Kind<'_>;
}

/// A type-erased description of an access handle.
pub enum Kind<'a> {
    /// Eager shared access; ignored by the broker and always readable.
    Frozen,
    /// Direct access to a known store.
    Store(&'a dyn DynStore),
    /// A field view with a known root store; selection runs on each borrow.
    Mapping(&'a dyn DynMapping),
}

/// Describes an access handle that the broker can acquire for writing.
pub trait DynAccessMut: DynAccess {
    /// Describes how the broker should acquire this handle for writing.
    fn kind_mut(&self) -> KindMut<'_>;
}

/// A type-erased description of a writable access handle.
pub enum KindMut<'a> {
    /// Direct access to a known store.
    Store(&'a dyn DynStoreMut),
    /// A writable field view with a known root store.
    Mapping(&'a dyn DynMappingMut),
}

/// The identity shared by a store's handles and leases.
///
/// The ID comes from the store's backing allocation, whose address must remain
/// fixed while any handle or lease refers to it. Moving a handle does not move
/// that allocation. All handles and leases for the same store return the same ID.
///
/// Reading the ID neither acquires a lock nor grants permission to borrow data.
/// A copied ID does not keep the allocation alive.
pub trait StoreIdentity: private::Sealed {
    /// Returns the backing allocation's address as an opaque key.
    ///
    /// Use the same canonical, nonzero-sized allocation for typed and erased
    /// handles and for leases. The address may be reused after destruction.
    fn id(&self) -> StoreId;
}

/// Owns a store's backend lock until dropped.
///
/// A lease keeps the backing allocation alive and pinned at its original address,
/// independently of the handle that acquired it. Dropping the lease unlocks the
/// store before releasing its ownership. Its identity is the store's ID, not the
/// address of the lease itself.
///
/// Leases must support moving to another thread and unlocking there. A lease is
/// not a typed borrow guard: the session separately checks conflicts between
/// [`Read`](crate::access::Read) and [`Write`](crate::access::Write) guards.
pub trait Lease: StoreIdentity + Send {}

/// A store handle that can retain its backing allocation and acquire read leases.
///
/// The backing allocation must stay alive and pinned in memory while any handle
/// or lease refers to it. This is an address-stability contract; it does not
/// require handles to use [`Pin`](std::pin::Pin), and the handles may move freely.
/// Pinning the allocation also does not freeze its contents or pin elements of
/// containers stored inside it. References into mutable data need borrow guards.
///
/// Retained handles and leases provide the ownership needed to keep cached store
/// identities valid. Weak bookkeeping must check liveness before using an entry.
pub trait DynStore: DynAccess + StoreIdentity + 'static {
    /// Retains the same backing allocation without acquiring a lock.
    ///
    /// This must be cheap and nonblocking. The returned handle keeps both the
    /// allocation and its identity valid, and can acquire leases later.
    fn retain(&self) -> Arc<dyn DynStore + Sync + 'static>;

    /// Attempts to acquire a read lease without waiting.
    /// On success, the lease retains this allocation, reports the same ID, and
    /// releases the lock on drop. On failure, no new lease is held.
    fn try_read(&self) -> Result<Box<dyn Lease>>;
}

/// A store handle that can also acquire exclusive write leases.
pub trait DynStoreMut: DynStore + DynAccessMut {
    /// Retains the same allocation with its write capability, without locking it.
    /// The identity and lifetime guarantees are those of [`DynStore::retain`].
    fn retain_mut(&self) -> Arc<dyn DynStoreMut + Sync + 'static>;

    /// Attempts to acquire a write lease without waiting.
    /// On success, the lease retains this allocation and excludes other backend
    /// leases until dropped. Its ID is the same as this handle's ID.
    fn try_write(&self) -> Result<Box<dyn Lease>>;
}

/// A field view whose root store is already known.
///
/// The mapping retains its root. Acquisition locks that store; it does not
/// evaluate the selector or preserve the identity of a selected field. Selection
/// runs on each borrow and may fail or produce a different result after mutation.
/// For example, selecting the first vector element follows the current first
/// element, rather than keeping an earlier element alive after removal.
///
/// Once a borrow succeeds, its guard protects the selected reference for the
/// guard's lifetime. Conflicting borrows are checked across the whole store.
pub trait DynMapping: DynAccess + 'static {
    /// The root store whose lease should be acquired to enable this mapping.
    fn root(&self) -> &dyn DynStore;
}

/// A known mapping that also supports exclusive access to its selected target.
pub trait DynMappingMut: DynMapping + DynAccessMut {
    /// The same root store, with the capability to acquire write leases.
    fn root_mut(&self) -> &dyn DynStoreMut;
}
