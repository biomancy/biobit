//! Nonblocking acquisition of storage leases.

mod session;
#[cfg(test)]
mod tests;

pub use session::{Mode, Session};

use crate::access::borrow::Entry;
use crate::access::{
    DynAccess, DynAccessMut, DynStore, DynStoreMut, Error, Kind, KindMut, Result, StoreId,
};
use session::Permission;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

struct BrokerState {
    // Only Acquiring exposes this cell after taking the nonblocking gate.
    entries: RefCell<HashMap<StoreId, Weak<Entry>>>,
    acquiring: AtomicBool,
    // Shared by all frozen guards in this broker, outside store permissions.
    frozen: Entry,
}

impl Default for BrokerState {
    fn default() -> Self {
        Self {
            entries: RefCell::default(),
            acquiring: AtomicBool::new(false),
            frozen: Entry::frozen(),
        }
    }
}

// SAFETY: bookkeeping is accessed only through Acquiring, which exclusively
// owns the atomic gate. Session destruction never accesses this cell.
// The frozen entry is immutable, read-only, and carries no backend lease.
// This private Sync implementation permits Arc ownership to move between
// threads; public broker handles deliberately remain !Sync.
unsafe impl Sync for BrokerState {}

/// Coordinates store leases and borrowing across sessions.
/// Clones share leases and borrow accounting. Independent operations can create
/// separate brokers.
///
/// A broker is [`Send`] but not [`Sync`]; overlapping use of clones is checked
/// by one nonblocking gate and returns [`Error::Busy`].
#[derive(Clone, Default)]
pub struct Broker {
    state: Arc<BrokerState>,
    not_sync: PhantomData<Cell<()>>,
}

// The gate owner itself is !Sync: its RefCell access cannot be shared with
// another thread, even though the protected allocation uses Arc ownership.
struct Acquiring(Arc<BrokerState>, PhantomData<Cell<()>>);

impl Acquiring {
    fn try_new(state: &Arc<BrokerState>) -> Result<Self> {
        state
            .acquiring
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| Error::Busy)?;
        Ok(Self(state.clone(), PhantomData))
    }

    fn entries(&self) -> &RefCell<HashMap<StoreId, Weak<Entry>>> {
        &self.0.entries
    }
}

impl Drop for Acquiring {
    fn drop(&mut self) {
        self.0.acquiring.store(false, Ordering::Release);
    }
}

impl Broker {
    /// Creates an independent broker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquires store permissions without waiting for backend leases.
    ///
    /// Requests for the same store are merged before locking; write permission
    /// wins when both are requested. Existing leases of this broker are shared,
    /// but a live read lease cannot be upgraded to writing.
    ///
    /// Failed attempts release their newly acquired leases. Concurrent or
    /// reentrant acquisition on this broker returns [`Error::Busy`]. Mapping
    /// selectors run only when borrowed; frozen values need no lease.
    pub fn acquire(
        &self,
        reads: &[&dyn DynAccess],
        writes: &[&dyn DynAccessMut],
    ) -> Result<Session> {
        let acquiring = Acquiring::try_new(&self.state)?;

        enum Request<'a> {
            Read(&'a dyn DynStore),
            Write(&'a dyn DynStoreMut),
        }

        let mut requests = BTreeMap::new();
        for access in writes {
            let store = match access.kind_mut() {
                KindMut::Store(store) => store,
                KindMut::Mapping(mapping) => mapping.root_mut(),
            };
            requests.insert(store.id(), Request::Write(store));
        }
        for access in reads {
            let store = match access.kind() {
                Kind::Frozen => continue,
                Kind::Store(store) => store,
                Kind::Mapping(mapping) => mapping.root(),
            };
            requests.entry(store.id()).or_insert(Request::Read(store));
        }

        // Stage permissions locally. Any failure drops the leases acquired by
        // this attempt before releasing the gate or changing shared bookkeeping.
        let mut permissions = BTreeMap::new();
        for (id, request) in requests {
            let mode = match &request {
                Request::Read(_) => Mode::Read,
                Request::Write(_) => Mode::Write,
            };
            let existing = acquiring
                .entries()
                .borrow()
                .get(&id)
                .and_then(Weak::upgrade);
            let entry = match existing {
                Some(entry) => {
                    if mode == Mode::Write && entry.mode() != Mode::Write {
                        return Err(Error::UpgradeDenied);
                    }
                    entry
                }
                None => {
                    let lease = match request {
                        Request::Read(store) => store.try_read()?,
                        Request::Write(store) => store.try_write()?,
                    };
                    Arc::new(Entry::new(lease, mode))
                }
            };
            // A read request stays read-only even when sharing a write lease.
            permissions.insert(id, Permission { mode, entry });
        }

        {
            let mut entries = acquiring.entries().borrow_mut();
            // Session drops only release owners; prune expired bookkeeping here.
            entries.retain(|_, entry| entry.strong_count() > 0);
            for (id, permission) in &permissions {
                entries.insert(*id, Arc::downgrade(&permission.entry));
            }
        }
        Ok(Session {
            _broker: self.clone(),
            permissions,
        })
    }

    /// Counts stores with live leases shared by this broker.
    /// Returns [`Error::Busy`] during acquisition or another inspection.
    /// Bookkeeping holds only weak references and does not retain leases.
    pub fn active_stores(&self) -> Result<usize> {
        let acquiring = Acquiring::try_new(&self.state)?;
        Ok(acquiring
            .entries()
            .borrow()
            .values()
            .filter(|w| w.strong_count() > 0)
            .count())
    }
}
