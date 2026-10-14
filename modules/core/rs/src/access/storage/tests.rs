use crate::access::{ArcRwLock, Broker, DynStore, DynStoreMut, Error, Mode, StoreIdentity};
use std::sync::Arc;

type Heap<T> = ArcRwLock<parking_lot::RawRwLock, T>;

#[test]
fn lease_retains_the_allocation_after_its_handle_is_dropped() {
    let value = Arc::new(());
    let store = Heap::new(value.clone());
    let lease = store.try_read().unwrap();

    drop(store);
    assert_eq!(Arc::strong_count(&value), 2);
    drop(lease);
    assert_eq!(Arc::strong_count(&value), 1);
}

#[test]
fn erased_handles_share_the_allocation_and_retain_it_without_a_lease() {
    let value = Arc::new(());
    let store = Heap::new(value.clone());
    let id = store.id();
    let reader = store.retain();
    let writer = store.retain_mut();

    assert_eq!(reader.id(), id);
    assert_eq!(writer.id(), id);
    assert!(Arc::ptr_eq(&reader, &store.retain()));
    assert!(Arc::ptr_eq(&writer, &store.retain_mut()));
    drop(store);

    let retained_reader = reader.retain();
    let retained_writer = writer.retain_mut();
    assert!(Arc::ptr_eq(&reader, &retained_reader));
    assert!(Arc::ptr_eq(&writer, &retained_writer));
    assert!(Arc::ptr_eq(&reader, &retained_writer.retain()));
    drop(reader);
    drop(writer);
    assert_eq!(Arc::strong_count(&value), 2);

    // Owning read handles do not take read leases or prevent exclusive locking.
    let lease = retained_writer.try_write().unwrap();
    drop(retained_reader);
    drop(retained_writer);
    assert_eq!(Arc::strong_count(&value), 2);
    drop(lease);
    assert_eq!(Arc::strong_count(&value), 1);
}

#[test]
fn leases_enforce_backend_exclusion_and_unlock_on_drop() {
    let store = Heap::new(1);
    let other = store.clone();
    let reader = other.retain();
    let writer = other.retain_mut();
    assert_eq!(store.id(), other.id());

    // Shared leases coexist, but exclude a writer until both are released.
    let first = store.try_read().unwrap();
    let second = reader.try_read().unwrap();
    assert!(matches!(writer.try_write(), Err(Error::Busy)));
    drop(first);
    assert!(matches!(other.try_write(), Err(Error::Busy)));
    drop(second);

    // An exclusive lease excludes either permission, then restores access.
    let lease = writer.try_write().unwrap();
    assert!(matches!(store.try_read(), Err(Error::Busy)));
    assert!(matches!(reader.try_read(), Err(Error::Busy)));
    assert!(matches!(other.try_write(), Err(Error::Busy)));
    drop(lease);
    assert!(store.try_read().is_ok());
}

#[test]
fn erased_stores_support_broker_acquisition_after_typed_handles_are_dropped() {
    let store = Heap::new(1);
    let id = store.id();
    let erased = store.retain_mut();
    drop(store);

    let broker = Broker::new();
    let session = broker.acquire(&[], &[erased.as_ref()]).unwrap();
    drop(erased);
    assert_eq!(session.mode(id), Some(Mode::Write));
    assert_eq!(broker.active_stores().unwrap(), 1);
    drop(session);
    assert_eq!(broker.active_stores().unwrap(), 0);
}
