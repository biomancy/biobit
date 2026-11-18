use crate::access::private;
use crate::access::{
    Access, AccessMut, DynAccess, DynAccessMut, DynStore, DynStoreMut, Error, Kind, KindMut, Lease,
    Mode, Read, Result, Session, Store, StoreId, StoreIdentity, StoreMut, Write,
};
use lock_api::{RawRwLock, RwLock};
use std::ptr::NonNull;
use std::sync::{Arc, Weak};

struct Allocation<L: RawRwLock, T> {
    lock: RwLock<L, T>,
    // Erased handles point directly to this allocation. Its borrowed trait
    // methods use the weak owner to retain another handle without a wrapper.
    owner: Weak<Self>,
}

impl<L: RawRwLock, T> Allocation<L, T> {
    fn retain_allocation(&self) -> Arc<Self> {
        // Every reference to this private allocation comes from a live Arc.
        // It is never exposed during construction or after its owners drop.
        self.owner
            .upgrade()
            .expect("a live store allocation has an owning handle")
    }
}

/// Shared heap storage protected by a reader/writer lock supplied by `L`.
///
/// The backing allocation stays alive and pinned at a fixed address while any
/// handle or lease retains it. Handles themselves may move freely. Clones share
/// the allocation and lock; their [`StoreIdentity::id`] is its address, computed
/// without a counter or an extra stored ID.
///
/// Pass a store to a broker to acquire it, then borrow its value through the
/// session's guards. [`DynStore::retain`] and [`DynStoreMut::retain_mut`] retain
/// the same allocation for type-erased locking without allocating a wrapper or
/// taking a lease. Leases keep the allocation alive even after all handles drop.
///
/// Store capabilities require `L` and `T` to be `Send + Sync`, and
/// both of `L`'s raw guard types to be `Send` so leases can be dropped on
/// another thread. For `parking_lot::RawRwLock`, enable the `send_guard` feature.
pub struct ArcRwLock<L: RawRwLock + 'static, T: 'static> {
    inner: Arc<Allocation<L, T>>,
}

impl<L: RawRwLock + 'static, T> ArcRwLock<L, T> {
    /// Allocates the value behind an initially unlocked reader/writer lock.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new_cyclic(|owner| Allocation {
                lock: RwLock::new(value),
                owner: owner.clone(),
            }),
        }
    }
}

impl<L: RawRwLock + 'static, T> Clone for ArcRwLock<L, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<L: RawRwLock + 'static, T> private::Sealed for ArcRwLock<L, T> {}

impl<L: RawRwLock + 'static, T> StoreIdentity for ArcRwLock<L, T> {
    fn id(&self) -> StoreId {
        StoreId::from_ptr(Arc::as_ptr(&self.inner))
    }
}

impl<L, T> DynAccess for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn kind(&self) -> Kind<'_> {
        Kind::Store(self.inner.as_ref())
    }
}

impl<L, T> DynAccessMut for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn kind_mut(&self) -> KindMut<'_> {
        KindMut::Store(self.inner.as_ref())
    }
}

impl<L, T> Access for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    type Target = T;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, T>> {
        session.read(self)
    }
}

impl<L, T> AccessMut for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn borrow_mut<'a>(&'a self, session: &'a Session) -> Result<Write<'a, T>> {
        session.write(self)
    }
}

impl<L, T> DynStore for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn retain(&self) -> Arc<dyn DynStore + Sync + 'static> {
        self.inner.clone()
    }

    fn try_read(&self) -> Result<Box<dyn Lease>> {
        HeapLease::acquire(self.inner.clone(), Mode::Read)
    }
}

impl<L, T> DynStoreMut for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn retain_mut(&self) -> Arc<dyn DynStoreMut + Sync + 'static> {
        self.inner.clone()
    }

    fn try_write(&self) -> Result<Box<dyn Lease>> {
        HeapLease::acquire(self.inner.clone(), Mode::Write)
    }
}

impl<L, T> Store for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn ptr(&self) -> NonNull<T> {
        // SAFETY: the pointer addresses the value inside the retained allocation.
        unsafe { NonNull::new_unchecked(self.inner.lock.data_ptr()) }
    }
}

impl<L, T> StoreMut for ArcRwLock<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
}

impl<L: RawRwLock, T> private::Sealed for Allocation<L, T> {}

impl<L: RawRwLock, T> StoreIdentity for Allocation<L, T> {
    fn id(&self) -> StoreId {
        StoreId::from_ptr(std::ptr::from_ref(self))
    }
}

impl<L, T> DynAccess for Allocation<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn kind(&self) -> Kind<'_> {
        Kind::Store(self)
    }
}

impl<L, T> DynAccessMut for Allocation<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn kind_mut(&self) -> KindMut<'_> {
        KindMut::Store(self)
    }
}

impl<L, T> DynStore for Allocation<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn retain(&self) -> Arc<dyn DynStore + Sync + 'static> {
        self.retain_allocation()
    }

    fn try_read(&self) -> Result<Box<dyn Lease>> {
        HeapLease::acquire(self.retain_allocation(), Mode::Read)
    }
}

impl<L, T> DynStoreMut for Allocation<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn retain_mut(&self) -> Arc<dyn DynStoreMut + Sync + 'static> {
        self.retain_allocation()
    }

    fn try_write(&self) -> Result<Box<dyn Lease>> {
        HeapLease::acquire(self.retain_allocation(), Mode::Write)
    }
}

enum RawGuard<L: RawRwLock> {
    Read(L::SharedGuard),
    Write(L::ExclusiveGuard),
}

struct HeapLease<L: RawRwLock, T> {
    owner: Arc<Allocation<L, T>>,
    guard: Option<RawGuard<L>>,
}

impl<L, T> HeapLease<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn acquire(owner: Arc<Allocation<L, T>>, mode: Mode) -> Result<Box<dyn Lease>> {
        // SAFETY: this backend never creates lock_api guards or exposes the
        // RwLock. Only a successfully acquired lease can unlock it.
        let raw = unsafe { owner.lock.raw() };
        let guard = match mode {
            Mode::Read => raw.try_lock_shared().map(RawGuard::Read),
            Mode::Write => raw.try_lock_exclusive().map(RawGuard::Write),
        }
        .ok_or(Error::Busy)?;
        Ok(Box::new(Self {
            owner,
            guard: Some(guard),
        }))
    }
}

impl<L: RawRwLock, T> private::Sealed for HeapLease<L, T> {}

impl<L: RawRwLock, T> StoreIdentity for HeapLease<L, T> {
    fn id(&self) -> StoreId {
        StoreId::from_ptr(Arc::as_ptr(&self.owner))
    }
}

impl<L, T> Lease for HeapLease<L, T>
where
    L: RawRwLock<SharedGuard: Send, ExclusiveGuard: Send> + Send + Sync,
    T: Send + Sync,
{
}

impl<L: RawRwLock, T> Drop for HeapLease<L, T> {
    fn drop(&mut self) {
        let guard = self.guard.take().expect("a live lease holds a lock");
        // SAFETY: the guard state comes from this lock and matches its acquired
        // mode. The lease retains the allocation and outlives its data borrows.
        unsafe {
            match guard {
                RawGuard::Read(state) => self.owner.lock.force_unlock_read(state),
                RawGuard::Write(state) => self.owner.lock.force_unlock_write(state),
            }
        }
    }
}
