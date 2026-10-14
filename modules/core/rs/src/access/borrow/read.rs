//! A shared reference associated with a read entry.

use super::Entry;
use crate::access::Result;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

/// A shared borrow tied to its owner and, for locked values, its session.
/// It can cross threads when `T: Sync`.
pub struct Read<'a, T: ?Sized> {
    pub(in crate::access) data: NonNull<T>,
    pub(in crate::access) entry: &'a Entry,
    pub(in crate::access) marker: PhantomData<&'a T>,
}

// SAFETY: either an ordinary shared reference protects the target, or a session
// retains its transferable lease and shared borrow claim. Moving this guard
// exposes &T, so it follows that reference's Send bound. It remains !Sync.
unsafe impl<T: ?Sized + Sync> Send for Read<'_, T> {}

impl<'a, T: ?Sized> Read<'a, T> {
    /// Selects a field and transfers its entry and any claim. Failure releases it.
    pub fn try_map<U: ?Sized, F>(self, f: F) -> Result<Read<'a, U>>
    where
        F: for<'b> FnOnce(&'b T) -> Result<&'b U>,
    {
        // SAFETY: the owner and any borrow claim remain alive during selection.
        let root: &'a T = unsafe { self.data.as_ref() };
        let selected = f(root)?;
        let mapped = Read {
            data: NonNull::from(selected),
            entry: self.entry,
            marker: PhantomData,
        };
        std::mem::forget(self);
        Ok(mapped)
    }
}

impl<T: ?Sized> Deref for Read<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: the owner's lifetime and any borrow claim keep the selection valid.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> Drop for Read<'_, T> {
    fn drop(&mut self) {
        // SAFETY: a write-mode entry carries exactly one shared claim, which
        // mapping transfers rather than duplicates. Read-mode entries, including
        // the broker's frozen entry, require no accounting. References to the
        // target cannot outlive this guard.
        unsafe { self.entry.release_read() };
    }
}
