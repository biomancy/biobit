//! Shared backend and Python protocol methods for read-only mappings.

use pyo3::IntoPyObjectExt;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::{PyMapping, PyMappingMethods};

use super::PyIterator;

/// Backend for read-only Python mapping operations. Implementations supply
/// traversal and lookup; default methods define shared get and comparison behavior.
pub trait PyMappingBackend: Send + Sync {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter_keys(&self) -> PyIterator;

    fn lookup(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Option<Py<PyAny>>>;

    fn contains(&self, key: &Bound<'_, PyAny>) -> bool;

    fn getitem(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.lookup(py, key)?
            .ok_or_else(|| PyKeyError::new_err((key.clone().unbind(),)))
    }

    fn get(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        default: Option<Py<PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        Ok(self
            .lookup(py, key)?
            .unwrap_or_else(|| default.unwrap_or_else(|| py.None())))
    }

    fn compare(
        &self,
        py: Python<'_>,
        other: &Bound<'_, PyAny>,
        op: CompareOp,
    ) -> PyResult<Py<PyAny>> {
        if !matches!(op, CompareOp::Eq | CompareOp::Ne) {
            return Ok(py.NotImplemented());
        }
        let Ok(other) = other.cast::<PyMapping>() else {
            return Ok(py.NotImplemented());
        };
        let mut equal = other.len()? == self.len();
        if equal {
            let mut keys = self.iter_keys();
            while let Some(key) = keys.next(py)? {
                let value = self
                    .lookup(py, key.bind(py))?
                    .expect("an indexed key disappeared");
                let candidate = match other.get_item(key.bind(py)) {
                    Ok(value) => value,
                    Err(error) if error.is_instance_of::<PyKeyError>(py) => {
                        equal = false;
                        break;
                    }
                    Err(error) => return Err(error),
                };
                if !value.bind(py).eq(candidate)? {
                    equal = false;
                    break;
                }
            }
        }
        let result = if matches!(op, CompareOp::Eq) {
            equal
        } else {
            !equal
        };
        Ok(result.into_bound_py_any(py)?.unbind())
    }
}

/// Temporarily delegates keys(), values(), and items() to collections.abc views.
/// These Python objects retain the index and provide repeatable iteration, length,
/// membership, and the keys/items set operations without native view classes.
/// Values and items are currently generated through Python iteration and indexed
/// lookup rather than direct Rust record traversal.
///
/// TODO: Replace this delegation with native Rust view wrappers sharing a retained,
/// type-erased index backend. The wrappers must preserve the public view behavior
/// while creating iterators directly over Rust data. This removes the Python view
/// constructor/generator calls; converting results still requires Python's C API.
#[doc(hidden)]
pub fn mapping_view<'py, T>(py: Python<'py>, index: T, name: &str) -> PyResult<Py<PyAny>>
where
    T: IntoPyObject<'py>,
    T::Error: Into<PyErr>,
{
    Ok(PyModule::import(py, "collections.abc")?
        .getattr(name)?
        .call1((index.into_bound_py_any(py)?,))?
        .unbind())
}

/// Exposes a PyMappingBackend implementation through Python mapping methods.
#[macro_export]
macro_rules! py_mapping_methods {
    ($class:ident) => {
        #[::pyo3::pymethods]
        impl $class {
            fn __len__(&self) -> usize {
                $crate::utils::collections::PyMappingBackend::len(self)
            }

            fn __iter__(&self) -> $crate::utils::collections::PyIterator {
                $crate::utils::collections::PyMappingBackend::iter_keys(self)
            }

            fn __getitem__(
                &self,
                py: ::pyo3::Python<'_>,
                key: &::pyo3::Bound<'_, ::pyo3::types::PyAny>,
            ) -> ::pyo3::PyResult<::pyo3::Py<::pyo3::types::PyAny>> {
                $crate::utils::collections::PyMappingBackend::getitem(self, py, key)
            }

            #[pyo3(signature = (key, default=None))]
            fn get(
                &self,
                py: ::pyo3::Python<'_>,
                key: &::pyo3::Bound<'_, ::pyo3::types::PyAny>,
                default: Option<::pyo3::Py<::pyo3::types::PyAny>>,
            ) -> ::pyo3::PyResult<::pyo3::Py<::pyo3::types::PyAny>> {
                $crate::utils::collections::PyMappingBackend::get(self, py, key, default)
            }

            fn __contains__(&self, key: &::pyo3::Bound<'_, ::pyo3::types::PyAny>) -> bool {
                $crate::utils::collections::PyMappingBackend::contains(self, key)
            }

            fn keys(
                slf: ::pyo3::PyRef<'_, Self>,
                py: ::pyo3::Python<'_>,
            ) -> ::pyo3::PyResult<::pyo3::Py<::pyo3::types::PyAny>> {
                $crate::utils::collections::mapping_view(py, slf, "KeysView")
            }

            fn values(
                slf: ::pyo3::PyRef<'_, Self>,
                py: ::pyo3::Python<'_>,
            ) -> ::pyo3::PyResult<::pyo3::Py<::pyo3::types::PyAny>> {
                $crate::utils::collections::mapping_view(py, slf, "ValuesView")
            }

            fn items(
                slf: ::pyo3::PyRef<'_, Self>,
                py: ::pyo3::Python<'_>,
            ) -> ::pyo3::PyResult<::pyo3::Py<::pyo3::types::PyAny>> {
                $crate::utils::collections::mapping_view(py, slf, "ItemsView")
            }

            fn __richcmp__(
                &self,
                py: ::pyo3::Python<'_>,
                other: &::pyo3::Bound<'_, ::pyo3::types::PyAny>,
                op: ::pyo3::basic::CompareOp,
            ) -> ::pyo3::PyResult<::pyo3::Py<::pyo3::types::PyAny>> {
                $crate::utils::collections::PyMappingBackend::compare(self, py, other, op)
            }
        }
    };
}
