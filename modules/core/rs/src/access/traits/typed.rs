use super::{DynAccess, DynAccessMut, DynMapping, DynMappingMut, DynStore, DynStoreMut};
use crate::access::{
    Fallible, Infallible, Read, Result, Session, StaticMapping, StaticMappingMut, Write,
};
use std::ptr::NonNull;

/// An access handle that can borrow its value through a session.
///
/// The broker acquires the required stores first. Borrowing then checks the
/// session's permissions and returns a guard that protects the selected value.
/// Mappings may run selectors as part of borrowing.
pub trait Access: DynAccess {
    /// The borrowed value's type.
    type Target: ?Sized + 'static;

    /// Borrows the target for reading without acquiring a new backend lock.
    ///
    /// The session must already permit reading the root store, and no conflicting
    /// write guard may exist. Borrow checks cover the whole store, including its
    /// field views. A selector can also fail when evaluating a mapping.
    ///
    /// Keep the returned guard alive while using the reference. Frozen access
    /// needs no store permission and always succeeds.
    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, Self::Target>>;
}

/// An access handle that can also borrow its value exclusively.
pub trait AccessMut: Access + DynAccessMut {
    /// Borrows the target for writing without acquiring a new backend lock.
    /// The session must have write permission, and no conflicting read or write
    /// guard may exist. The returned guard protects the borrow until dropped.
    fn borrow_mut<'a>(&'a self, session: &'a Session) -> Result<Write<'a, Self::Target>>;
}

/// A typed field view backed by a known store.
/// Its selector runs on each borrow under the store's borrow protection.
pub trait Mapping: DynMapping + Access {}

/// A typed mapping that also supports exclusive access to its target.
pub trait MappingMut: Mapping + DynMappingMut + AccessMut {}

impl<M: DynMapping + Access + ?Sized> Mapping for M {}
impl<M: Mapping + DynMappingMut + AccessMut + ?Sized> MappingMut for M {}

/// An owning handle to data in a store with a fixed backing address.
///
/// The backing allocation must stay alive and pinned in memory while any handle
/// or lease refers to it. Cloning or moving a handle does not move the allocation;
/// clones share its lock and address-derived ID. This contract does not require
/// the handle itself to use [`Pin`](std::pin::Pin).
///
/// The target's location is stable, but its contents can change under a write
/// guard. Elements inside a vector, for example, are not pinned merely because
/// the vector belongs to a store. Field mappings retain the store and select
/// their data when borrowed, rather than caching mutable field addresses.
pub trait Store: DynStore + Access<Target: Send + Sync> + Clone + 'static {
    /// Returns the target pointer without acquiring a lease or borrow claim.
    /// Its address stays fixed while an owning handle or lease retains the
    /// allocation. The pointer alone grants no access: dereferencing requires
    /// the appropriate lease and borrow claim, with Rust's aliasing rules upheld.
    fn ptr(&self) -> NonNull<Self::Target>;

    /// Selects a field, retaining this store in a static mapping.
    /// The selector runs on each borrow, so later borrows may select different
    /// data after mutation. Use `map(|value| value)` to view the whole target.
    fn map<T: ?Sized + 'static, F>(self, selector: F) -> StaticMapping<Self, Infallible<F>>
    where
        Self: Sync,
        F: for<'a> Fn(&'a Self::Target) -> &'a T + Send + Sync + 'static,
    {
        StaticMapping::new(self, Infallible::new(selector))
    }

    /// Selects a field with a fallible selector, retaining this store.
    /// Acquisition needs only the store; selection may fail when borrowed.
    fn try_map<T: ?Sized + 'static, F>(self, selector: F) -> StaticMapping<Self, Fallible<F>>
    where
        Self: Sync,
        F: for<'a> Fn(&'a Self::Target) -> Result<&'a T> + Send + Sync + 'static,
    {
        StaticMapping::new(self, Fallible::new(selector))
    }
}

/// A store whose target permits mutation under a write lease and borrow claim.
/// Its backing address and identity obey the same contract as [`Store`].
pub trait StoreMut: Store + DynStoreMut + AccessMut {
    /// Selects a writable field, retaining this store in a static mapping.
    /// Both selectors should identify the same logical field.
    fn map_mut<T: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> StaticMappingMut<Self, Infallible<FR, FW>>
    where
        Self: Sync,
        FR: for<'a> Fn(&'a Self::Target) -> &'a T + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut Self::Target) -> &'a mut T + Send + Sync + 'static,
    {
        StaticMappingMut::new(self, Infallible::new_mut(read, write))
    }

    /// Selects a writable field with fallible read and write selectors.
    /// Both selectors should identify the same logical field.
    fn try_map_mut<T: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> StaticMappingMut<Self, Fallible<FR, FW>>
    where
        Self: Sync,
        FR: for<'a> Fn(&'a Self::Target) -> Result<&'a T> + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut Self::Target) -> Result<&'a mut T> + Send + Sync + 'static,
    {
        StaticMappingMut::new(self, Fallible::new_mut(read, write))
    }
}
