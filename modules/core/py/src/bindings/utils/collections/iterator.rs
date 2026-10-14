//! Python iteration over owned Rust iterators or iterators borrowing a retained source.
//!
//! Python can retain an iterator after dropping the object it came from. We must
//! therefore retain the Rust source too, usually through a cloned Frozen handle.
//! Frozen keeps the collection's allocation alive, but iterating it still borrows
//! the handle: owning a `'static` source does not turn that borrow into `'static`.
//! self_cell stores the retained source and its borrowed iterator together safely.
//!
//! The Rust iterator preserves traversal state between calls. This matters for
//! maps and sets: recreating an iterator and calling nth(position) on each step
//! would repeatedly scan earlier entries and make a complete traversal quadratic.
//! IteratorAdapter converts just the next item, using the Python token supplied
//! by __next__. PyIterator exposes that traversal through Python's protocol.

use std::any::Any;

use pyo3::prelude::*;

/// Builds a Python iterator from an owned source, a borrowed Rust traversal,
/// and an item converter. The source binding is a borrow of the stored owner;
/// it lets callers select different traversals of the same source type.
///
/// A plain generic builder `for<'a> FnOnce(&'a S) -> I` has one fixed return type
/// I, whereas a borrowed iterator's type depends on 'a. The macro creates and
/// boxes the adapter inside the builder, whose return type is BoxedIterator<'a>.
/// This expresses the lifetime relationship without a factory trait or requiring
/// callers to name or box their concrete iterator types. Traversal is constructed
/// once; item conversion runs lazily on each subsequent next() call.
#[macro_export]
macro_rules! py_iterator {
    ($source:expr, |$source_ref:ident| $iter:expr, $convert:expr $(,)?) => {
        $crate::utils::collections::PyIterator::from_source($source, move |$source_ref| {
            $crate::utils::collections::adapt($iter, $convert)
        })
    };
}

// Internal protocol used by the Python wrapper. IteratorAdapter bridges ordinary
// Rust iterators to this interface without attaching Python on every step.
#[doc(hidden)]
pub trait PyIteratorBackend: Send + Sync {
    fn next(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>>;
    fn size_hint(&self) -> (usize, Option<usize>);
}

struct IteratorAdapter<I, F> {
    iter: I,
    convert: F,
}

impl<I, F> PyIteratorBackend for IteratorAdapter<I, F>
where
    I: Iterator + Send + Sync,
    F: FnMut(Python<'_>, I::Item) -> PyResult<Py<PyAny>> + Send + Sync,
{
    fn next(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.iter
            .next()
            .map(|item| (self.convert)(py, item))
            .transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Internal adapter used by py_iterator!. Conversion is lazy; an error consumes
/// its item, and subsequent requests continue with the next Rust item.
#[doc(hidden)]
pub fn adapt<'a, I, F>(iter: I, convert: F) -> BoxedIterator<'a>
where
    I: Iterator + Send + Sync + 'a,
    F: FnMut(Python<'_>, I::Item) -> PyResult<Py<PyAny>> + Send + Sync + 'a,
{
    Box::new(IteratorAdapter {
        iter: iter.fuse(),
        convert,
    })
}

/// Erases the concrete iterator and converter types while preserving their borrow.
#[doc(hidden)]
pub type BoxedIterator<'a> = Box<dyn PyIteratorBackend + 'a>;

// This is an owner and an iterator borrowing that owner, not two independent
// iterators. self_cell places the owner in a stable allocation, constructs the
// dependent from a borrow of it, and drops the dependent before the owner. Moving
// PyIterator therefore does not invalidate the borrow. Any erases the source type
// so one Python class can hold different collections; from_source recovers it
// only during construction, with no downcast on subsequent iteration steps.
self_cell::self_cell! {
    struct IteratorState {
        owner: Box<dyn Any + Send + Sync>,
        #[not_covariant]
        dependent: BoxedIterator,
    }
}

/// Retains a Rust source and exposes its iterator through Python's protocol.
/// Returned Python objects must own or retain their data independently: keeping
/// the source here only guarantees access while this iterator remains alive.
#[pyclass(name = "_Iterator", module = "biobit.rs.core")]
pub struct PyIterator {
    state: IteratorState,
}

impl PyIterator {
    /// Internal constructor used by py_iterator!. The builder borrows the stored
    /// source and returns a boxed backend carrying the same borrowing lifetime.
    #[doc(hidden)]
    pub fn from_source<S, Build>(source: S, build: Build) -> Self
    where
        S: Send + Sync + 'static,
        Build: for<'a> FnOnce(&'a S) -> BoxedIterator<'a>,
    {
        Self {
            state: IteratorState::new(Box::new(source), move |source| {
                let source = source
                    .downcast_ref::<S>()
                    .expect("retained source type changed");
                build(source)
            }),
        }
    }

    pub fn next(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.state.with_dependent_mut(|_, iter| iter.next(py))
    }

    /// An owned iterator needs no separate source to retain.
    pub fn from_iter<I, F>(iter: I, convert: F) -> Self
    where
        I: Iterator + Send + Sync + 'static,
        F: FnMut(Python<'_>, I::Item) -> PyResult<Py<PyAny>> + Send + Sync + 'static,
    {
        Self::from_source((), move |_| adapt(iter, convert))
    }
}

#[pymethods]
impl PyIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.next(py)
    }

    fn __length_hint__(&self) -> usize {
        self.state.with_dependent(|_, iter| iter.size_hint().0)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use biobit_core_rs::access::Frozen;

    #[test]
    fn borrowed_iterator_retains_and_releases_frozen_source() {
        let source = Arc::new(BTreeSet::from([1, 2]));
        let weak = Arc::downgrade(&source);
        let frozen = Frozen::from_arc(source);
        // Python-visible traversal is covered by the index iterator tests. Here
        // we check the Rust allocation's lifetime without initializing Python.
        let iterator = py_iterator!(frozen.clone(), |source| source.iter(), |_, _| {
            unreachable!("this test does not consume items")
        });
        drop(frozen);
        assert!(weak.upgrade().is_some());

        drop(iterator);
        assert!(weak.upgrade().is_none());
    }
}
