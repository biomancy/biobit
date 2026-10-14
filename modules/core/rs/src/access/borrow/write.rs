//! An exclusive borrow claim, transferable to a selected field.

use super::Entry;
use crate::access::Result;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// An exclusive borrow tied to its session. It can cross threads when `T: Send`.
pub struct Write<'a, T: ?Sized> {
    pub(in crate::access) data: NonNull<T>,
    pub(in crate::access) entry: &'a Entry,
    pub(in crate::access) marker: PhantomData<&'a mut T>,
}

// SAFETY: the session retains a transferable lease, and the atomic ledger
// grants this guard exclusive access until it drops. Moving it exposes &mut T
// and follows that reference's Send bound. It deliberately remains !Sync.
unsafe impl<T: ?Sized + Send> Send for Write<'_, T> {}

impl<'a, T: ?Sized> Write<'a, T> {
    /// Transfers this borrow claim to a selected field. Failure releases it.
    pub fn try_map<U: ?Sized, F>(mut self, f: F) -> Result<Write<'a, U>>
    where
        F: for<'b> FnOnce(&'b mut T) -> Result<&'b mut U>,
    {
        // SAFETY: this guard owns the exclusive claim, transferred on success.
        let root: &'a mut T = unsafe { self.data.as_mut() };
        let data = NonNull::from(f(root)?);
        let entry = self.entry;
        std::mem::forget(self);
        Ok(Write {
            data,
            entry,
            marker: PhantomData,
        })
    }
}

impl<T: ?Sized> Deref for Write<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: this guard retains the selection's exclusive borrow claim.
        unsafe { self.data.as_ref() }
    }
}
impl<T: ?Sized> DerefMut for Write<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the exclusive claim prevents any other borrow of the store.
        unsafe { self.data.as_mut() }
    }
}

impl<T: ?Sized> Drop for Write<'_, T> {
    fn drop(&mut self) {
        // SAFETY: this guard owns the exclusive claim; its borrowed references
        // cannot outlive the guard. Mapping transfers rather than duplicates it.
        unsafe { self.entry.release_write() };
    }
}
