//! Typed, nonblocking access to owned storage.
//!
//! A [`Broker`] acquires storage leases. Its [`Session`] checks store-wide
//! borrowing and constructs typed [`Read`] and [`Write`] guards. Borrow through
//! handles with [`Access::borrow`] and [`AccessMut::borrow_mut`], passing the session.
//!
//! Stores own their data. [`Store::map`] and [`StoreMut::map_mut`] create field
//! mappings that retain the store and select their target on each borrow.
//! [`Store::try_map`] and [`StoreMut::try_map_mut`] accept fallible selectors.
//! Acquisition locks the root store without evaluating selectors or caching
//! field addresses. New mappings can borrow through an existing session when
//! it already has the required store permission.
//!
//! [`Selector`] and [`SelectorMut`] describe borrowed selection independently of
//! locking. Closure adapters preserve their captures' Clone bounds, and [`Then`]
//! composes them without allocation. [`StaticMapping`] retains a typed store;
//! [`ErasedMapping`] shares a mapping with its root and selector types erased.
//! Additional selection over an erased mapping stays inline in [`Mapped`] or
//! [`MappedMut`]; erase the extended mapping explicitly when it needs sharing.
//!
//! Store handles and leases implement [`StoreIdentity`]. Their [`StoreId`] is the
//! address of the backing allocation, which stays alive and pinned in memory
//! while retained. Handles may move freely. IDs do not own the allocation and
//! may be reused after it is destroyed. Cached associations must retain owners
//! or validate weak references. Pinning a store does not pin container elements.
//!
//! Each typed access trait extends its erased counterpart, so concrete handles,
//! generic handles, and typed trait objects can register directly with the broker.
//! Erased traits expose no typed borrowing or downcast interface.
//! [`Frozen`] retains selections for eager shared access without a broker or
//! session. It also implements [`Access`] for unconditional reads through any
//! session; the broker ignores frozen acquisition requests.
//!
//! Brokers and sessions are `Send`, not `Sync`: ownership can move between threads,
//! while references to them cannot be shared across threads. Cloned brokers share
//! leases and borrow accounting. A nonblocking acquisition gate protects broker
//! bookkeeping and returns [`Error::Busy`] on overlapping acquisition.
//! Borrowing uses the session's permissions and needs no bookkeeping locks.
//! Backend leases must support unlocking on another thread; with `parking_lot`,
//! enable its `send_guard` feature. Borrow guards also remain non-`Sync`:
//! [`Read`] is `Send` when its target is `Sync`; [`Write`] is `Send` when its target
//! is `Send`. Acquisition remains synchronous and nonblocking.
//!
//! This is a runnable design draft, not an audited synchronization library.
//!
//! Borrow a store directly:
//! ```
//! use biobit_core_rs::access::*;
//! let store = ArcRwLock::<parking_lot::RawRwLock, _>::new(1);
//! let broker = Broker::new();
//! let session = broker.acquire(&[], &[&store]).unwrap();
//! *store.borrow_mut(&session).unwrap() = 2;
//! assert_eq!(*store.borrow(&session).unwrap(), 2);
//! ```
//!
//! Select a field while retaining its store:
//! ```
//! use biobit_core_rs::access::*;
//! let root = ArcRwLock::<parking_lot::RawRwLock, _>::new(vec![1]);
//! let first = root.try_map_mut(
//!     |values| values.first().ok_or(Error::Missing),
//!     |values| values.first_mut().ok_or(Error::Missing),
//! );
//! let broker = Broker::new();
//! let session = broker.acquire(&[], &[&first]).unwrap();
//! *first.borrow_mut(&session).unwrap() = 2;
//! assert_eq!(*first.borrow(&session).unwrap(), 2);
//! ```
//!
//! Chain static selectors, then erase the root and selector types:
//! ```
//! use biobit_core_rs::access::*;
//! let root = ArcRwLock::<parking_lot::RawRwLock, _>::new(vec![(1, 2)]);
//! let first = root
//!     .map(|values| values.as_slice())
//!     .try_map(|values| values.first().ok_or(Error::Missing));
//! // Static composition retains the original store without allocating.
//! // Erasure shares the mapping in an Arc; later selectors stay inline.
//! let second = first.erase().map(|pair| &pair.1);
//! // Explicitly erase again when the extended mapping needs to be shared.
//! let second: ErasedMapping<i32> = second.erase();
//! let broker = Broker::new();
//! let session = broker.acquire(&[&second], &[]).unwrap();
//! assert_eq!(*second.borrow(&session).unwrap(), 2);
//! ```
//!
//! Generic typed handles and typed trait objects register directly:
//! ```
//! use biobit_core_rs::access::*;
//! fn acquire<A: Access>(broker: &Broker, access: &A) -> Result<Session> {
//!     broker.acquire(&[access], &[])
//! }
//! fn acquire_mapping(broker: &Broker, mapping: &dyn Mapping<Target = i32>) -> Result<Session> {
//!     broker.acquire(&[mapping], &[])
//! }
//! fn read<'a>(
//!     session: &'a Session,
//!     mapping: &'a dyn Mapping<Target = i32>,
//! ) -> Result<Read<'a, i32>> {
//!     mapping.borrow(session)
//! }
//! ```
//! An erased broker handle cannot be borrowed through the typed API:
//! ```compile_fail
//! use biobit_core_rs::access::*;
//! fn read(session: &Session, access: &dyn DynAccess) {
//!     access.borrow(session).unwrap();
//! }
//! ```
//! Its mapping interface does not expose a downcast back to typed access:
//! ```compile_fail
//! use biobit_core_rs::access::*;
//! use std::any::Any;
//! fn recover(mapping: &dyn DynMapping) -> &dyn Any {
//!     mapping
//! }
//! ```
//!
//! Frozen values are selected eagerly and read through any session:
//! ```
//! use biobit_core_rs::access::*;
//! let x = Frozen::new(vec![1]);
//! let first = x.map(|v: &Vec<i32>| &v[0]);
//! let broker = Broker::new();
//! let session = broker.acquire(&[], &[]).unwrap();
//! assert_eq!(*first.borrow(&session).unwrap(), 1);
//! assert_eq!(broker.active_stores().unwrap(), 0);
//! ```
//! A read-only mapping does not gain write capability from its root:
//! ```compile_fail
//! use biobit_core_rs::access::*;
//! let root = ArcRwLock::<parking_lot::RawRwLock, _>::new(vec![1]);
//! let m = root.clone().map(|values| &values[0]);
//! let broker = Broker::new();
//! let session = broker.acquire(&[], &[&root]).unwrap();
//! m.borrow_mut(&session).unwrap();
//! ```
//! A guard cannot outlive its session:
//! ```compile_fail
//! use biobit_core_rs::access::*;
//! let root = ArcRwLock::<parking_lot::RawRwLock, _>::new(1);
//! let broker = Broker::new();
//! let read = {
//!     let session = broker.acquire(&[&root], &[]).unwrap();
//!     root.borrow(&session).unwrap()
//! };
//! println!("{}", *read);
//! ```
//! A shared guard can move to another thread while its session remains alive:
//! ```
//! use biobit_core_rs::access::*;
//! let root = ArcRwLock::<parking_lot::RawRwLock, _>::new(1);
//! let broker = Broker::new();
//! let session = broker.acquire(&[&root], &[]).unwrap();
//! let read = root.borrow(&session).unwrap();
//! std::thread::scope(|scope| {
//!     scope.spawn(move || println!("{}", *read));
//! });
//! ```
//! A broker reference cannot be shared between threads:
//! ```compile_fail
//! use biobit_core_rs::access::Broker;
//! fn require_sync<T: Sync>() {}
//! require_sync::<Broker>();
//! ```
//! A session reference cannot be shared between threads:
//! ```compile_fail
//! use biobit_core_rs::access::Session;
//! fn require_sync<T: Sync>() {}
//! require_sync::<Session>();
//! ```
//! Borrow guards allow ownership transfer, not shared references across threads:
//! ```compile_fail
//! use biobit_core_rs::access::Read;
//! fn require_sync<T: Sync>() {}
//! require_sync::<Read<'_, i32>>();
//! ```
//! ```compile_fail
//! use biobit_core_rs::access::Write;
//! fn require_sync<T: Sync>() {}
//! require_sync::<Write<'_, i32>>();
//! ```
//! A shared guard to a non-`Sync` target cannot cross threads:
//! ```compile_fail
//! use biobit_core_rs::access::Read;
//! fn require_send<T: Send>() {}
//! require_send::<Read<'_, std::cell::Cell<i32>>>();
//! ```
//! An exclusive guard to a non-`Send` target cannot cross threads:
//! ```compile_fail
//! use biobit_core_rs::access::Write;
//! fn require_send<T: Send>() {}
//! require_send::<Write<'_, std::rc::Rc<i32>>>();
//! ```
//! A selector cannot return a reference through a temporary interior lock:
//! ```compile_fail
//! use biobit_core_rs::access::*;
//! let root = Frozen::new(std::sync::Mutex::new(1));
//! let mapped = root.map(|m| &*m.lock().unwrap());
//! ```
#![deny(unsafe_op_in_unsafe_fn)]

mod borrow;
mod broker;
mod error;
mod frozen;
mod identity;
mod mapping;
mod storage;
mod traits;

mod private {
    pub trait Sealed {}
}

pub use borrow::{Read, Write};
pub use broker::{Broker, Mode, Session};
pub use error::{Error, Result};
pub use frozen::Frozen;
pub use identity::StoreId;
pub use mapping::{
    ErasedMapping, ErasedMappingMut, Fallible, Infallible, Mapped, MappedMut, Selector,
    SelectorMut, StaticMapping, StaticMappingMut, Then,
};
pub use storage::ArcRwLock;
pub use traits::{
    Access, AccessMut, DynAccess, DynAccessMut, DynMapping, DynMappingMut, DynStore, DynStoreMut,
    Kind, KindMut, Lease, Mapping, MappingMut, Store, StoreIdentity, StoreMut,
};
