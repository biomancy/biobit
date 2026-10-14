//! Reusable collection adapters for Python bindings.

mod iterable;
mod iterator;
mod mapping;

pub use crate::{py_iterator, py_mapping_methods};
pub use iterable::Iterable;
pub use iterator::PyIterator;
pub use mapping::PyMappingBackend;

// Public only so exported macros can reach their implementation across crates.
#[doc(hidden)]
pub use iterator::{BoxedIterator, PyIteratorBackend, adapt};

#[doc(hidden)]
pub use mapping::mapping_view;
