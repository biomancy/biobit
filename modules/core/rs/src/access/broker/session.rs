//! Acquired store permissions and runtime checks for typed borrow guards.

use super::Broker;
use crate::access::borrow::Entry;
use crate::access::{Error, Frozen, Read, Result, Store, StoreId, StoreMut, Write};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

/// The permission held by a session for one store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// Allows shared borrows.
    Read,
    /// Allows shared or exclusive borrows, subject to borrow conflicts.
    Write,
}

pub(super) struct Permission {
    pub(super) mode: Mode,
    pub(super) entry: Arc<Entry>,
}

/// Owns acquired permissions and checks conflicts between store borrows.
/// Access handles request typed guards through
/// [`Access::borrow`](crate::access::Access::borrow) or
/// [`AccessMut::borrow_mut`](crate::access::AccessMut::borrow_mut), passing this session.
/// Conflicts include guards held by other sessions of the same broker.
/// Sessions are [`Send`] but not [`Sync`]: move ownership between threads rather
/// than sharing a session reference. Each permission retains its store's lease.
pub struct Session {
    pub(super) _broker: Broker,
    pub(super) permissions: BTreeMap<StoreId, Permission>,
}

impl Session {
    /// Constructs a shared store guard under an acquired lease.
    /// The borrow claim covers the whole store, including disjoint fields.
    pub(in crate::access) fn read<'a, S: Store>(
        &'a self,
        store: &'a S,
    ) -> Result<Read<'a, S::Target>> {
        let entry = self.entry(store.id(), false)?;
        let data = store.ptr();
        entry.claim_read()?;
        // The pointer belongs to the entry's retained allocation. After claiming
        // the borrow, construction is infallible and transfers the claim to Read.
        Ok(Read {
            data,
            entry,
            marker: PhantomData,
        })
    }

    /// Constructs an exclusive store guard under an acquired write lease.
    pub(in crate::access) fn write<'a, S: StoreMut>(
        &'a self,
        store: &'a S,
    ) -> Result<Write<'a, S::Target>> {
        let entry = self.entry(store.id(), true)?;
        let data = store.ptr();
        entry.claim_write()?;
        // StoreMut permits mutable access to this pointer. The entry retains the
        // write lease, and construction immediately transfers the exclusive claim.
        Ok(Write {
            data,
            entry,
            marker: PhantomData,
        })
    }

    /// Constructs an unconditional shared guard using the broker's frozen entry.
    pub(in crate::access) fn frozen<'a, T: ?Sized + Send + Sync + 'static>(
        &'a self,
        value: &'a Frozen<T>,
    ) -> Read<'a, T> {
        Read {
            data: NonNull::from(&**value),
            entry: &self._broker.state.frozen,
            marker: PhantomData,
        }
    }

    pub fn mode(&self, id: StoreId) -> Option<Mode> {
        self.permissions.get(&id).map(|p| p.mode)
    }

    fn entry(&self, id: StoreId, write: bool) -> Result<&Entry> {
        let p = self.permissions.get(&id).ok_or(Error::NotAcquired)?;
        if write && p.mode != Mode::Write {
            return Err(Error::NotAcquired);
        }
        Ok(&p.entry)
    }
}
