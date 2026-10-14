use crate::access::{Access, AccessMut, ArcRwLock, Broker, Error};
use std::panic::{AssertUnwindSafe, catch_unwind};

type Heap<T> = ArcRwLock<parking_lot::RawRwLock, T>;

#[test]
fn mapped_guards_retain_the_whole_store_claim() {
    let root = Heap::new((1, 2));
    let session = Broker::new().acquire(&[], &[&root]).unwrap();

    // Selecting one field retains the exclusive claim on its entire store.
    let mut field = root
        .borrow_mut(&session)
        .unwrap()
        .try_map(|value| Ok(&mut value.0))
        .unwrap();
    *field = 3;
    assert!(matches!(root.borrow(&session), Err(Error::BorrowConflict)));
    drop(field);

    // A shared selection likewise prevents a subsequent exclusive borrow.
    let field = root
        .borrow(&session)
        .unwrap()
        .try_map(|value| Ok(&value.1))
        .unwrap();
    assert!(matches!(
        root.borrow_mut(&session),
        Err(Error::BorrowConflict)
    ));
    drop(field);
    assert_eq!(*root.borrow_mut(&session).unwrap(), (3, 2));
}

#[test]
fn failed_read_selection_releases_its_claim() {
    let root = Heap::new(1);
    let session = Broker::new().acquire(&[], &[&root]).unwrap();

    let result = root
        .borrow(&session)
        .unwrap()
        .try_map::<i32, _>(|_| Err(Error::Missing));
    assert!(matches!(result, Err(Error::Missing)));
    assert!(root.borrow_mut(&session).is_ok());
}

#[test]
fn panicking_write_selection_releases_its_claim() {
    let root = Heap::new(1);
    let session = Broker::new().acquire(&[], &[&root]).unwrap();

    let result = catch_unwind(AssertUnwindSafe(|| {
        root.borrow_mut(&session)
            .unwrap()
            .try_map::<i32, _>(|_| panic!("selector failed"))
    }));
    assert!(result.is_err());
    assert!(root.borrow_mut(&session).is_ok());
}

#[test]
fn transferred_guards_share_borrow_accounting_across_threads() {
    use std::sync::Barrier;

    let root = Heap::new((1, 2));
    let broker = Broker::new();
    let first = broker.acquire(&[], &[&root]).unwrap();
    let second = broker.acquire(&[], &[&root]).unwrap();
    let held = Barrier::new(2);
    let release = Barrier::new(2);

    // Move a mapped writer to another thread. Other sessions cannot access
    // even a disjoint field until that thread releases the whole-store claim.
    let mut writer = root
        .borrow_mut(&first)
        .unwrap()
        .try_map(|value| Ok(&mut value.0))
        .unwrap();
    std::thread::scope(|scope| {
        let (held, release) = (&held, &release);
        scope.spawn(move || {
            *writer = 3;
            held.wait();
            release.wait();
            drop(writer);
        });
        held.wait();
        let read = root.borrow(&second);
        let write = root.borrow_mut(&second);
        release.wait();
        assert!(matches!(read, Err(Error::BorrowConflict)));
        assert!(matches!(write, Err(Error::BorrowConflict)));
    });

    // Readers can coexist across threads. Releasing one must not permit a
    // writer while the reader transferred to the worker is still alive.
    let reader = root
        .borrow(&first)
        .unwrap()
        .try_map(|value| Ok(&value.1))
        .unwrap();
    let local_reader = root.borrow(&second).unwrap();
    std::thread::scope(|scope| {
        let (held, release) = (&held, &release);
        scope.spawn(move || {
            held.wait();
            release.wait();
            assert_eq!(*reader, 2);
            drop(reader);
        });
        held.wait();
        drop(local_reader);
        let write = root.borrow_mut(&second);
        release.wait();
        assert!(matches!(write, Err(Error::BorrowConflict)));
    });
    assert_eq!(*root.borrow_mut(&second).unwrap(), (3, 2));
}
