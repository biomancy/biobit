use crate::access::private;
use crate::access::{Access, DynAccess, Kind, Read, Result, Session};
use std::any::Any;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;

/// Shared, immutable access to an owned value or one of its fields.
/// The target is Send + Sync + 'static so handles can cross threads.
///
/// [`Frozen`] is a shallow guarantee: fields with interior mutability, including
/// locking stores, retain their own mutation rules. All [`Frozen`] derivatives
/// through [`Frozen::map`] and [`Frozen::try_map`] are eager and share the same
/// underlying data.
///
/// Generic [`Access`] borrowing provides a fast path for frozen values - their
/// read-only access is always available with minimal session overhead.
pub struct Frozen<T: ?Sized + Send + Sync + 'static> {
    data: NonNull<T>,
    owner: Arc<dyn Any + Send + Sync>,
}

// SAFETY: the owner retains the selection's allocation and T permits
// shared access from any thread. Selectors cannot return temporary-guard refs.
unsafe impl<T: ?Sized + Send + Sync> Send for Frozen<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for Frozen<T> {}

impl<T: Send + Sync + 'static> Frozen<T> {
    /// Allocates a value for shared access.
    pub fn new(value: T) -> Self {
        let owner = Arc::new(value);
        Self::from_arc(owner)
    }
}

impl<T: ?Sized + Send + Sync + 'static> Frozen<T> {
    /// Retains an existing shared allocation.
    pub fn from_arc(inner: Arc<T>) -> Self {
        let data = NonNull::from(inner.as_ref());
        Self {
            data,
            owner: Arc::new(inner),
        }
    }

    /// Selects a field immediately and retains its owner's allocation.
    pub fn map<U: ?Sized + Send + Sync + 'static, F>(&self, selector: F) -> Frozen<U>
    where
        F: for<'a> FnOnce(&'a T) -> &'a U,
    {
        Frozen {
            data: NonNull::from(selector(self)),
            owner: self.owner.clone(),
        }
    }

    /// Selects a field immediately, propagating the selector's error.
    pub fn try_map<U: ?Sized + Send + Sync + 'static, E, F>(
        &self,
        selector: F,
    ) -> std::result::Result<Frozen<U>, E>
    where
        F: for<'a> FnOnce(&'a T) -> std::result::Result<&'a U, E>,
    {
        Ok(Frozen {
            data: NonNull::from(selector(self)?),
            owner: self.owner.clone(),
        })
    }

    /// Iterates shared references as frozen views retaining the same owner.
    ///
    /// The iterator borrows this handle, but each yielded view independently
    /// retains the owner's allocation and can outlive both the iterator and this
    /// handle. Elements are neither cloned nor looked up again. Use [`Self::map`]
    /// first to select a collection field within a larger value.
    ///
    /// Only traversals yielding references borrowed from the collection are
    /// supported; computed values and map entries such as `(&K, &V)` are not.
    pub fn iter_frozen<U>(&self) -> impl Iterator<Item = Frozen<U>> + '_
    where
        U: ?Sized + Send + Sync + 'static,
        for<'a> &'a T: IntoIterator<Item = &'a U>,
    {
        self.as_ref().into_iter().map(move |item| Frozen {
            data: NonNull::from(item),
            owner: self.owner.clone(),
        })
    }
}

impl<T: ?Sized + Send + Sync> Clone for Frozen<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            owner: self.owner.clone(),
        }
    }
}

impl<T: ?Sized + Send + Sync> Deref for Frozen<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: construction retains the allocation and stable shared target.
        unsafe { self.data.as_ref() }
    }
}

impl<T: Send + Sync + 'static> From<T> for Frozen<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: ?Sized + Send + Sync> AsRef<T> for Frozen<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized + Send + Sync + Debug> Debug for Frozen<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(formatter)
    }
}

impl<T: ?Sized + Send + Sync + PartialEq> PartialEq for Frozen<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T: ?Sized + Send + Sync + Eq> Eq for Frozen<T> {}

impl<T: ?Sized + Send + Sync + PartialOrd> PartialOrd for Frozen<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T: ?Sized + Send + Sync + Ord> Ord for Frozen<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized + Send + Sync + Hash> Hash for Frozen<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<T: ?Sized + Send + Sync> private::Sealed for Frozen<T> {}

impl<T: ?Sized + Send + Sync> DynAccess for Frozen<T> {
    fn kind(&self) -> Kind<'_> {
        Kind::Frozen
    }
}

impl<T: ?Sized + Send + Sync> Access for Frozen<T> {
    type Target = T;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, T>> {
        Ok(session.frozen(self))
    }
}

#[cfg(test)]
mod tests {
    use super::Frozen;
    use crate::access::{
        Access, AccessMut, ArcRwLock, Broker, Error, Mode, Read, Session, StoreIdentity, StoreMut,
    };
    use std::sync::Arc;

    type Heap<T> = ArcRwLock<parking_lot::RawRwLock, T>;

    #[test]
    fn eager_selection_retains_its_owner() {
        let owner = Arc::new(vec![String::from("retained")]);
        let parent = Frozen::from_arc(owner.clone());
        let field = parent.map(|value| value[0].as_str());

        drop(parent);
        assert_eq!(&*field, "retained");
        assert_eq!(Arc::strong_count(&owner), 2);
        drop(field);
        assert_eq!(Arc::strong_count(&owner), 1);
    }

    #[test]
    fn eager_navigation_supports_unsized_values() {
        let root = Frozen::from_arc(Arc::<[i32]>::from(vec![1]));
        let field = root
            .try_map(|value| value.first().ok_or(Error::Missing))
            .unwrap();

        drop(root);
        assert_eq!(*field, 1);
    }

    #[test]
    fn iterated_views_retain_and_release_their_owner() {
        let owner = Arc::new(vec![String::from("first"), String::from("second")]);
        let weak = Arc::downgrade(&owner);
        let parent = Frozen::from_arc(owner);
        let collection = parent.map(Vec::as_slice);
        let mut iter = collection.iter_frozen();
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        assert!(iter.next().is_none());
        assert!(std::ptr::eq(first.as_ref(), &collection[0]));
        assert!(std::ptr::eq(second.as_ref(), &collection[1]));

        drop(iter);
        drop(collection);
        drop(parent);
        assert_eq!(first.as_str(), "first");
        assert_eq!(second.as_str(), "second");
        assert!(weak.upgrade().is_some());
        drop(first);
        assert!(weak.upgrade().is_some());
        drop(second);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn generic_access_reads_frozen_values_without_leases() {
        fn first<'a, A: Access<Target = [i32]> + ?Sized>(
            value: &'a A,
            session: &'a Session,
        ) -> Read<'a, i32> {
            value
                .borrow(session)
                .unwrap()
                .try_map(|values| values.first().ok_or(Error::Missing))
                .unwrap()
        }

        let value = Frozen::from_arc(Arc::<[i32]>::from(vec![7, 9]));
        let broker = Broker::new();
        let acquired = broker.acquire(&[&value], &[]).unwrap();
        let empty = broker.acquire(&[], &[]).unwrap();

        // Concrete and typed Access trait objects both work. An empty session is enough:
        // the frozen owner itself guarantees the references' shared lifetime.
        let read = first(&value, &acquired);
        let typed: &dyn Access<Target = [i32]> = &value;
        let other = first(typed, &empty);
        std::thread::scope(|scope| {
            scope.spawn(move || assert_eq!(*read, 7));
        });
        assert_eq!(*other, 7);
        assert_eq!(broker.active_stores().unwrap(), 0);
    }

    #[test]
    fn eager_frozen_selection_can_enter_writable_storage() {
        let leaf = Heap::new(vec![1]);
        let parent = Frozen::new(vec![leaf.clone()]);
        let selected = parent
            .try_map(|values| values.first().ok_or(Error::Missing))
            .unwrap();
        let store = (*selected).clone();
        drop(selected);
        drop(parent);
        let field = store.try_map_mut(
            |values: &Vec<i32>| values.first().ok_or(Error::Missing),
            |values: &mut Vec<i32>| values.first_mut().ok_or(Error::Missing),
        );

        // Eager navigation finishes before the selected store enters the broker.
        let broker = Broker::new();
        let session = broker.acquire(&[], &[&field]).unwrap();
        assert_eq!(session.mode(leaf.id()), Some(Mode::Write));
        assert_eq!(broker.active_stores().unwrap(), 1);
        *field.borrow_mut(&session).unwrap() = 2;
        assert_eq!(*field.borrow(&session).unwrap(), 2);
    }

    #[test]
    fn eager_frozen_selection_remains_fixed_after_parent_changes() {
        let parent = Heap::new(Frozen::new(vec![7, 9]));
        let broker = Broker::new();
        let first = broker.acquire(&[&parent], &[]).unwrap();
        let field = Frozen::try_map(&parent.borrow(&first).unwrap(), |values| {
            values.first().ok_or(Error::Missing)
        })
        .unwrap();
        drop(first);

        // The eager selection retains its frozen owner after the parent lease ends.
        // An empty session can read it while another broker replaces the parent.
        let empty = broker.acquire(&[], &[]).unwrap();
        let writer = Broker::new().acquire(&[], &[&parent]).unwrap();
        *parent.borrow_mut(&writer).unwrap() = Frozen::new(vec![20]);
        assert_eq!(*field.borrow(&empty).unwrap(), 7);
        assert_eq!(broker.active_stores().unwrap(), 0);
        drop(writer);

        // Reading the parent again permits a new eager selection of its replacement.
        let fresh = broker.acquire(&[&parent], &[]).unwrap();
        let replacement = Frozen::try_map(&parent.borrow(&fresh).unwrap(), |values| {
            values.first().ok_or(Error::Missing)
        })
        .unwrap();
        assert_eq!(*replacement.borrow(&fresh).unwrap(), 20);
    }
}
