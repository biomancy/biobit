use pyo3::conversion::FromPyObjectOwned;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyString;

/// Extracts owned values from a Python iterable, preserving iteration errors.
///
/// PyO3's Vec<T> extraction requires the sequence protocol, so it rejects generators,
/// sets, frozensets, and mapping views. APIs annotated as Iterable should also
/// accept generators and iterable collection views as inputs. This adapter uses
/// the iterator protocol instead and eagerly collects the extracted values into
/// a Vec. Strings are rejected to avoid interpreting one scalar value as a
/// collection of characters.
pub struct Iterable<T>(Vec<T>);

impl<'py, T: FromPyObjectOwned<'py>> FromPyObject<'_, 'py> for Iterable<T> {
    type Error = PyErr;

    fn extract(value: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if value.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err(
                "expected an iterable of typed values, not str",
            ));
        }
        value
            .try_iter()?
            .map(|item| item?.extract::<T>().map_err(Into::into))
            .collect::<PyResult<Vec<_>>>()
            .map(Self)
    }
}

impl<T> IntoIterator for Iterable<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
