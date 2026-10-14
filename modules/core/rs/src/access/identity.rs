//! Identities for live storage allocations.
//!
//! Store IDs are the addresses of their backing allocations. They remain
//! stable while those allocations are alive, but may be reused after destruction.

/// Identifies a live store by its backing allocation's address.
///
/// Handles and leases for the same store share this ID. The ID itself does not
/// keep the store alive: cached associations must retain a handle or lease.
/// After the allocation is destroyed, another store may receive the same ID.
/// IDs are local to this process and are unsuitable as persistent identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreId(usize);

impl StoreId {
    /// Uses the data address of a store's canonical, nonzero-sized allocation.
    /// The address is an equality/ordering key and is never dereferenced.
    pub(super) fn from_ptr<T: ?Sized>(allocation: *const T) -> Self {
        Self(allocation.addr())
    }
}
