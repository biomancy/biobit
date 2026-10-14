use crate::access::{
    Access, AccessMut, ArcRwLock, Broker, DynMapping, ErasedMappingMut, Error, Infallible,
    MappedMut, Result, Selector, SelectorMut, StaticMappingMut, StoreIdentity, StoreMut, Then,
};
use std::sync::Mutex;

type Heap<T> = ArcRwLock<parking_lot::RawRwLock, T>;

#[test]
fn composed_maps_remain_clonable_and_share_the_original_store() {
    let root = Heap::new(vec![(String::from("first"), String::from("second"))]);
    let write = root
        .clone()
        .map_mut(|values| values.as_slice(), |values| values.as_mut_slice())
        .try_map_mut(
            |values| values.first().ok_or(Error::Missing),
            |values| values.first_mut().ok_or(Error::Missing),
        )
        .map_mut(|pair| &pair.1, |pair| &mut pair.1);
    let cloned_write = write.clone();
    let read = write
        .read_only()
        .map(|text| text.as_str())
        .try_map(|text| text.get(..3).ok_or(Error::Missing));
    let cloned_read = read.clone();
    assert_eq!(cloned_write.root().id(), root.id());
    assert_eq!(cloned_read.root().id(), root.id());

    let broker = Broker::new();
    let session = broker.acquire(&[&cloned_read], &[&cloned_write]).unwrap();
    *cloned_write.borrow_mut(&session).unwrap() = String::from("changed");
    assert_eq!(&*cloned_read.borrow(&session).unwrap(), "cha");
}

#[test]
fn consuming_read_only_conversion_accepts_non_clone_selectors() {
    let root = Heap::new(vec![String::from("value")]);
    let (read_state, write_state) = (Mutex::new(()), Mutex::new(()));
    let read = root
        .try_map_mut(
            move |values| {
                drop(read_state.lock().unwrap());
                values.first().ok_or(Error::Missing)
            },
            move |values| {
                drop(write_state.lock().unwrap());
                values.first_mut().ok_or(Error::Missing)
            },
        )
        .into_read_only()
        .map(|text| text.as_str())
        .erase();
    let cloned = read.clone();
    let broker = Broker::new();
    let session = broker.acquire(&[&cloned], &[]).unwrap();
    assert_eq!(&*cloned.borrow(&session).unwrap(), "value");

    let root = Heap::new(vec![String::from("other")]);
    let (read_state, write_state) = (Mutex::new(()), Mutex::new(()));
    let read = ErasedMappingMut::from_store(root)
        .try_map_mut(
            move |values| {
                drop(read_state.lock().unwrap());
                values.first().ok_or(Error::Missing)
            },
            move |values| {
                drop(write_state.lock().unwrap());
                values.first_mut().ok_or(Error::Missing)
            },
        )
        .into_read_only()
        .map(|text| text.as_str());
    let session = broker.acquire(&[&read], &[]).unwrap();
    assert_eq!(&*read.borrow(&session).unwrap(), "other");
}

#[derive(Clone)]
struct At(usize);

impl<T> Selector<Vec<T>> for At {
    type Target = T;

    fn select<'a>(&self, values: &'a Vec<T>) -> Result<&'a T> {
        values.get(self.0).ok_or(Error::Missing)
    }
}

impl<T> SelectorMut<Vec<T>> for At {
    fn select_mut<'a>(&self, values: &'a mut Vec<T>) -> Result<&'a mut T> {
        values.get_mut(self.0).ok_or(Error::Missing)
    }
}

#[test]
fn custom_selectors_work_with_typed_and_erased_roots() {
    let root = Heap::new(vec![String::from("before")]);
    let typed = StaticMappingMut::new(
        root.clone(),
        Then::new(
            Infallible::new_mut(
                |values: &Vec<String>| values,
                |values: &mut Vec<String>| values,
            ),
            At(0),
        ),
    )
    .map_mut(|text| text.as_str(), |text| text.as_mut_str());
    let erased = MappedMut::new(ErasedMappingMut::from_store(root.clone()), At(0))
        .map_mut(|text| text.as_str(), |text| text.as_mut_str());
    let cloned = erased.clone();
    assert_eq!(DynMapping::root(&erased).id(), root.id());

    let broker = Broker::new();
    let session = broker.acquire(&[&typed], &[&cloned]).unwrap();
    cloned.borrow_mut(&session).unwrap().make_ascii_uppercase();
    assert_eq!(&*typed.borrow(&session).unwrap(), "BEFORE");
    assert_eq!(
        &*cloned.into_read_only().borrow(&session).unwrap(),
        "BEFORE"
    );
}
