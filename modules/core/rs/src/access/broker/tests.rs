use crate::access::{
    Access, AccessMut, ArcRwLock, Broker, Error, Mapping, MappingMut, Mode, Store, StoreIdentity,
    StoreMut,
};

type Heap<T> = ArcRwLock<parking_lot::RawRwLock, T>;

#[test]
fn typed_handles_and_trait_objects_register_directly() {
    fn acquire<R: Access, W: AccessMut>(
        broker: &Broker,
        read: &R,
        write: &W,
    ) -> crate::access::Result<crate::access::Session> {
        broker.acquire(&[read], &[write])
    }

    let number = Heap::new(1);
    let text = Heap::new(String::from("before"));
    let broker = Broker::new();
    let session = acquire(&broker, &number, &text).unwrap();
    assert_eq!(*number.borrow(&session).unwrap(), 1);
    *text.borrow_mut(&session).unwrap() = String::from("after");
    drop(session);

    let number: &dyn Access<Target = i32> = &number;
    let text: &dyn AccessMut<Target = String> = &text;
    let session = broker.acquire(&[number], &[text]).unwrap();
    assert_eq!(*number.borrow(&session).unwrap(), 1);
    assert_eq!(&*text.borrow(&session).unwrap(), "after");
}

#[test]
fn failed_acquisition_releases_partial_leases() {
    let left = Heap::new(1);
    let right = Heap::new(2);
    let (first, blocked) = if left.id() < right.id() {
        (left, right)
    } else {
        (right, left)
    };
    let blocker = Broker::new().acquire(&[], &[&blocked]).unwrap();
    let broker = Broker::new();

    // The first lease is acquired before the second encounters another broker.
    assert!(matches!(
        broker.acquire(&[], &[&first, &blocked]),
        Err(Error::Busy)
    ));
    assert_eq!(broker.active_stores().unwrap(), 0);
    assert!(Broker::new().acquire(&[], &[&first]).is_ok());
    drop(blocker);
}

#[test]
fn overlapping_sessions_share_the_borrow_ledger() {
    let root = Heap::new(1);
    let mapping = root.clone().map_mut(|value| value, |value| value);
    let broker = Broker::new();
    let first = broker.acquire(&[], &[&root]).unwrap();
    let guard = root.borrow_mut(&first).unwrap();

    // Acquisition grants a store permission without claiming a mapped field.
    let second = broker.acquire(&[], &[&mapping]).unwrap();
    assert!(matches!(
        mapping.borrow(&second),
        Err(Error::BorrowConflict)
    ));
    drop(guard);
    *mapping.borrow_mut(&second).unwrap() = 2;
    assert_eq!(*root.borrow(&first).unwrap(), 2);
}

#[test]
fn mapping_selectors_run_only_when_borrowed() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    let root = Heap::new(Vec::<i32>::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let (read_calls, write_calls) = (calls.clone(), calls.clone());
    let (read_capture, write_capture) = (Mutex::new(()), Mutex::new(()));
    let missing = root
        .clone()
        .map_mut(|values| values.as_slice(), |values| values.as_mut_slice())
        .try_map_mut(
            move |value: &[i32]| {
                drop(read_capture.lock().unwrap());
                read_calls.fetch_add(1, Ordering::Relaxed);
                value.first().ok_or(Error::Missing)
            },
            move |value: &mut [i32]| {
                drop(write_capture.lock().unwrap());
                write_calls.fetch_add(1, Ordering::Relaxed);
                value.first_mut().ok_or(Error::Missing)
            },
        );
    let broker = Broker::new();

    for mode in [Mode::Read, Mode::Write] {
        let before = calls.load(Ordering::Relaxed);
        let session = match mode {
            Mode::Read => broker.acquire(&[&missing], &[]),
            Mode::Write => broker.acquire(&[], &[&missing]),
        }
        .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), before);
        assert_eq!(session.mode(root.id()), Some(mode));

        // A failed selector releases its borrow claim and preserves the lease.
        assert!(matches!(missing.borrow(&session), Err(Error::Missing)));
        if mode == Mode::Write {
            assert!(matches!(missing.borrow_mut(&session), Err(Error::Missing)));
            assert!(root.borrow_mut(&session).is_ok());
        }
        assert!(root.borrow(&session).unwrap().is_empty());
        assert!(matches!(
            Broker::new().acquire(&[], &[&root]),
            Err(Error::Busy)
        ));
        drop(session);
        assert!(Broker::new().acquire(&[], &[&root]).is_ok());
    }
}

#[test]
fn mappings_merge_read_and_write_permissions_for_the_same_store() {
    let root = Heap::new((1, 2));
    let first = root.clone().map(|value| &value.0);
    let second = root
        .clone()
        .map_mut(|value| value, |value| value)
        .map_mut(|value| &value.1, |value| &mut value.1)
        .erase();
    let first_view: &dyn Mapping<Target = i32> = &first;
    let second_view: &dyn MappingMut<Target = i32> = &second;

    // Static and erased trait objects expose the same known root. Merge their
    // permissions before locking; selectors run when borrowed.
    let broker = Broker::new();
    let session = broker
        .acquire(&[first_view, &second.read_only()], &[second_view])
        .unwrap();
    assert_eq!(session.mode(root.id()), Some(Mode::Write));
    assert_eq!(broker.active_stores().unwrap(), 1);
    assert_eq!(*first_view.borrow(&session).unwrap(), 1);
    *second_view.borrow_mut(&session).unwrap() = 3;
    assert_eq!(*root.borrow(&session).unwrap(), (1, 3));
}

#[test]
fn static_and_erased_mappings_support_non_clone_selectors() {
    let root = Heap::new(vec![1]);
    let id = root.id();
    let calls = std::sync::Mutex::new(0);
    let mapping = root
        .map(|values| values.as_slice())
        .try_map(move |values: &[i32]| {
            *calls.lock().unwrap() += 1;
            values.first().ok_or(Error::Missing)
        });

    // Composition keeps the original typed store, not an intermediate mapping.
    let retained_root: &Heap<Vec<i32>> = mapping.root();
    assert_eq!(retained_root.id(), id);

    let broker = Broker::new();
    let session = broker.acquire(&[&mapping], &[]).unwrap();
    assert_eq!(*mapping.borrow(&session).unwrap(), 1);
    drop(session);

    // Erasure consumes the static mapping; cloning shares its non-Clone selector.
    let mapping = mapping.erase().map(|value| value);
    let retained = mapping.clone();
    drop(mapping);
    let session = broker.acquire(&[&retained], &[]).unwrap();
    assert_eq!(*retained.borrow(&session).unwrap(), 1);
}

#[test]
fn overlapping_sessions_keep_their_requested_permissions() {
    let root = Heap::new(1);
    let broker = Broker::new();
    let reader = broker.acquire(&[&root], &[]).unwrap();
    assert!(matches!(
        broker.acquire(&[], &[&root]),
        Err(Error::UpgradeDenied)
    ));
    drop(reader);

    let writer = broker.acquire(&[], &[&root]).unwrap();
    let reader = broker.acquire(&[&root], &[]).unwrap();
    assert_eq!(reader.mode(root.id()), Some(Mode::Read));
    assert!(matches!(root.borrow_mut(&reader), Err(Error::NotAcquired)));
    let guard = root.borrow(&reader).unwrap();
    assert!(matches!(
        root.borrow_mut(&writer),
        Err(Error::BorrowConflict)
    ));
    drop(guard);
    *root.borrow_mut(&writer).unwrap() = 2;
    assert_eq!(*root.borrow(&reader).unwrap(), 2);
}

#[test]
fn sessions_and_erased_mappings_survive_thread_migration() {
    let root = Heap::new(vec![1]);
    let field = root
        .clone()
        .map_mut(|value| value.as_slice(), |value| value.as_mut_slice())
        .erase()
        .try_map_mut(
            |value| value.first().ok_or(Error::Missing),
            |value| value.first_mut().ok_or(Error::Missing),
        )
        .erase();
    let broker = Broker::new();
    let first = broker.acquire(&[], &[&field]).unwrap();
    let (worker_broker, worker_field) = (broker.clone(), field.clone());
    std::thread::spawn(move || {
        let second = worker_broker.acquire(&[], &[&worker_field]).unwrap();
        assert_eq!(*worker_field.borrow(&second).unwrap(), 1);
        drop(first);
        *worker_field.borrow_mut(&second).unwrap() = 2;
    })
    .join()
    .unwrap();

    // The last session and its backend lease were dropped by the worker.
    assert_eq!(broker.active_stores().unwrap(), 0);
    let fresh = broker.acquire(&[&field], &[]).unwrap();
    assert_eq!(*field.borrow(&fresh).unwrap(), 2);
    assert_eq!(*root.borrow(&fresh).unwrap(), vec![2]);
}

#[test]
fn concurrent_and_reentrant_acquisitions_return_busy() {
    use super::Acquiring;

    let root = Heap::new(1);
    let broker = Broker::new();
    let acquiring = Acquiring::try_new(&broker.state).unwrap();
    assert!(matches!(broker.acquire(&[], &[]), Err(Error::Busy)));
    assert!(matches!(broker.active_stores(), Err(Error::Busy)));

    let competing = broker.clone();
    std::thread::spawn(move || {
        assert!(matches!(competing.acquire(&[], &[]), Err(Error::Busy)));
        assert!(matches!(competing.active_stores(), Err(Error::Busy)));
    })
    .join()
    .unwrap();

    drop(acquiring);
    let session = broker.acquire(&[&root], &[]).unwrap();
    assert_eq!(*root.borrow(&session).unwrap(), 1);
}
