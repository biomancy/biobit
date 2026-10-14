//! Store acquisition, ownership, and typed borrowing.
//!
//! Erased traits let the broker acquire stores without knowing their value types.
//! Typed traits use the resulting session to create borrow guards. A backend
//! lease keeps a store alive and locked; a borrow guard additionally enforces
//! aliasing rules for the value being accessed.
//!
//! Store handles and leases share an address-derived identity. The backing store
//! remains pinned in memory while referenced, even though its handles can move.

mod erased;
mod typed;

pub use erased::{
    DynAccess, DynAccessMut, DynMapping, DynMappingMut, DynStore, DynStoreMut, Kind, KindMut,
    Lease, StoreIdentity,
};
pub use typed::{Access, AccessMut, Mapping, MappingMut, Store, StoreMut};
