use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ptr;
use pyo3::PyObject;
use std::ops::{Deref, DerefMut};

/// Wrapper for python objects types that implements by-pointer comparison.
#[repr(transparent)]
pub struct ByPyPointer(pub PyObject);

impl ByPyPointer {
    #[inline(always)]
    fn ptr(&self) -> *mut pyo3::ffi::PyObject {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn from_ref(r: &PyObject) -> &Self {
        unsafe { &*(r as *const PyObject as *const Self) }
    }
}

impl PartialEq for ByPyPointer {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.ptr() as *const (), other.ptr() as *const _)
    }
}
impl Eq for ByPyPointer {}

impl Ord for ByPyPointer {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.ptr() as *const ()).cmp(&(other.ptr() as *const ()))
    }
}

impl PartialOrd for ByPyPointer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Hash for ByPyPointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.ptr() as *const ()).hash(state)
    }
}

impl Deref for ByPyPointer {
    type Target = PyObject;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByPyPointer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
