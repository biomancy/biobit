//! Erased mappings share owning mappings through the public typed traits.
//! Additional selection stays inline until explicitly erased again.

use super::{Fallible, Infallible, Mapped, MappedMut};
use crate::access::private;
use crate::access::{
    Access, AccessMut, DynAccess, DynAccessMut, DynMapping, DynMappingMut, DynStore, DynStoreMut,
    Kind, KindMut, Mapping, MappingMut, Read, Result, Session, Store, StoreMut, Write,
};
use std::sync::Arc;

/// A shared mapping whose only type parameter is its borrowed target.
///
/// Cloning shares the existing root and selector state. Further mapping returns
/// an inline adapter; call its erase method to share that extended mapping.
pub struct ErasedMapping<T: ?Sized + 'static> {
    inner: Arc<dyn Mapping<Target = T> + Sync>,
}

/// A shared writable mapping whose only type parameter is its borrowed target.
///
/// The owning Arc permits sharing selector captures even when they are not Clone.
pub struct ErasedMappingMut<T: ?Sized + 'static> {
    inner: Arc<dyn MappingMut<Target = T> + Sync>,
}

impl<T: ?Sized> Clone for ErasedMapping<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized> Clone for ErasedMappingMut<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized> ErasedMapping<T> {
    /// Shares a mapping while erasing its concrete handle and selector types.
    pub fn new<M: Mapping<Target = T> + Sync + 'static>(mapping: M) -> Self {
        Self {
            inner: Arc::new(mapping),
        }
    }

    /// Creates an identity mapping over a store's whole target.
    pub fn from_store<R: Store<Target = T> + Sync>(root: R) -> Self {
        root.map(|value| value).erase()
    }

    /// Selects another field with an inline adapter, without allocating.
    pub fn map<U: ?Sized + 'static, F>(self, selector: F) -> Mapped<Self, Infallible<F>>
    where
        F: for<'a> Fn(&'a T) -> &'a U + Send + Sync + 'static,
    {
        Mapped::new(self, Infallible::new(selector))
    }

    /// Selects a field with a fallible inline adapter, without allocating.
    pub fn try_map<U: ?Sized + 'static, F>(self, selector: F) -> Mapped<Self, Fallible<F>>
    where
        F: for<'a> Fn(&'a T) -> Result<&'a U> + Send + Sync + 'static,
    {
        Mapped::new(self, Fallible::new(selector))
    }
}

impl<T: ?Sized> ErasedMappingMut<T> {
    /// Shares a writable mapping while erasing its concrete handle and selector types.
    pub fn new<M: MappingMut<Target = T> + Sync + 'static>(mapping: M) -> Self {
        Self {
            inner: Arc::new(mapping),
        }
    }

    /// Creates a writable identity mapping over a store's whole target.
    pub fn from_store<R: StoreMut<Target = T> + Sync>(root: R) -> Self {
        root.map_mut(|value| value, |value| value).erase()
    }

    /// Shares the same mapping with only read capability.
    pub fn read_only(&self) -> ErasedMapping<T> {
        ErasedMapping {
            inner: self.inner.clone(),
        }
    }

    /// Transfers ownership into a read-only handle without cloning the Arc.
    pub fn into_read_only(self) -> ErasedMapping<T> {
        ErasedMapping { inner: self.inner }
    }

    /// Selects a shared field with an inline adapter, without allocating.
    pub fn map<U: ?Sized + 'static, F>(self, selector: F) -> Mapped<ErasedMapping<T>, Infallible<F>>
    where
        F: for<'a> Fn(&'a T) -> &'a U + Send + Sync + 'static,
    {
        self.into_read_only().map(selector)
    }

    /// Selects a shared field with a fallible inline adapter.
    pub fn try_map<U: ?Sized + 'static, F>(
        self,
        selector: F,
    ) -> Mapped<ErasedMapping<T>, Fallible<F>>
    where
        F: for<'a> Fn(&'a T) -> Result<&'a U> + Send + Sync + 'static,
    {
        self.into_read_only().try_map(selector)
    }

    /// Selects a writable field with an inline adapter, without allocating.
    pub fn map_mut<U: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> MappedMut<Self, Infallible<FR, FW>>
    where
        FR: for<'a> Fn(&'a T) -> &'a U + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut T) -> &'a mut U + Send + Sync + 'static,
    {
        MappedMut::new(self, Infallible::new_mut(read, write))
    }

    /// Selects a writable field with fallible inline selectors.
    pub fn try_map_mut<U: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> MappedMut<Self, Fallible<FR, FW>>
    where
        FR: for<'a> Fn(&'a T) -> Result<&'a U> + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut T) -> Result<&'a mut U> + Send + Sync + 'static,
    {
        MappedMut::new(self, Fallible::new_mut(read, write))
    }
}

impl<T: ?Sized> private::Sealed for ErasedMapping<T> {}
impl<T: ?Sized> private::Sealed for ErasedMappingMut<T> {}

impl<T: ?Sized> DynAccess for ErasedMapping<T> {
    fn kind(&self) -> Kind<'_> {
        Kind::Mapping(self)
    }
}

impl<T: ?Sized> Access for ErasedMapping<T> {
    type Target = T;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, T>> {
        self.inner.borrow(session)
    }
}

impl<T: ?Sized> DynMapping for ErasedMapping<T> {
    fn root(&self) -> &dyn DynStore {
        self.inner.root()
    }
}

impl<T: ?Sized> DynAccess for ErasedMappingMut<T> {
    fn kind(&self) -> Kind<'_> {
        Kind::Mapping(self)
    }
}

impl<T: ?Sized> DynAccessMut for ErasedMappingMut<T> {
    fn kind_mut(&self) -> KindMut<'_> {
        KindMut::Mapping(self)
    }
}

impl<T: ?Sized> Access for ErasedMappingMut<T> {
    type Target = T;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, T>> {
        self.inner.borrow(session)
    }
}

impl<T: ?Sized> AccessMut for ErasedMappingMut<T> {
    fn borrow_mut<'a>(&'a self, session: &'a Session) -> Result<Write<'a, T>> {
        self.inner.borrow_mut(session)
    }
}

impl<T: ?Sized> DynMapping for ErasedMappingMut<T> {
    fn root(&self) -> &dyn DynStore {
        self.inner.root()
    }
}

impl<T: ?Sized> DynMappingMut for ErasedMappingMut<T> {
    fn root_mut(&self) -> &dyn DynStoreMut {
        self.inner.root_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::{ArcRwLock, Broker, Error, Mapping, MappingMut};
    use std::sync::Mutex;

    #[test]
    fn composed_trait_objects_share_non_clone_selectors_and_their_root() {
        let root = ArcRwLock::<parking_lot::RawRwLock, _>::new(vec![1]);
        let reads = Mutex::new(0);
        let writes = Mutex::new(0);
        let write = ErasedMappingMut::from_store(root)
            .try_map_mut(
                move |value: &Vec<i32>| {
                    *reads.lock().unwrap() += 1;
                    value.first().ok_or(Error::Missing)
                },
                move |value: &mut Vec<i32>| {
                    *writes.lock().unwrap() += 1;
                    value.first_mut().ok_or(Error::Missing)
                },
            )
            .erase();
        let read = write.read_only();
        let selector = Mutex::new(());
        let selection = read.map(move |value: &i32| {
            drop(selector.lock().unwrap());
            value
        });
        let selection: &dyn Mapping<Target = i32> = &selection;
        let write: &dyn MappingMut<Target = i32> = &write;

        // Both handles share the original selectors and allocation.
        // The composed read selection still conflicts with the mutable guard.
        let broker = Broker::new();
        let session = broker.acquire(&[selection], &[write]).unwrap();
        let mut guard = write.borrow_mut(&session).unwrap();
        assert!(matches!(
            selection.borrow(&session),
            Err(Error::BorrowConflict)
        ));
        *guard = 2;
        drop(guard);
        assert_eq!(*selection.borrow(&session).unwrap(), 2);
    }
}
