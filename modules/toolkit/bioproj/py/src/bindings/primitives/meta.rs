use biobit_bioproj_rs::{Meta, MetaVal, Text};
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyMapping, PyMappingMethods, PyString, PyTuple};

use crate::utils::error::{PyBioprojErr, PyBioprojResult};
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::{
    PyIterator, PyMappingBackend, py_iterator, py_mapping_methods,
};

/// A read-only Python mapping that retains the Rust owner of its metadata.
#[pyclass(skip_from_py_object, frozen, name = "Meta")]
#[derive(Clone, Debug)]
pub struct PyMeta(pub Frozen<Meta>);

frozen_wrapper!(PyMeta, Meta);

impl<'a, 'py> FromPyObject<'a, 'py> for PyMeta {
    type Error = PyBioprojErr;

    fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyBioprojResult<Self> {
        if let Ok(value) = value.cast::<Self>() {
            return Ok(Self(value.borrow().0.clone()));
        }
        let mapping = value.cast::<PyMapping>().map_err(PyErr::from)?;
        let mut entries = Vec::new();
        for item in mapping.items()?.iter() {
            let item = item.cast_into::<PyTuple>().map_err(PyErr::from)?;
            if item.len() != 2 {
                return Err(PyValueError::new_err(
                    "metadata entries must contain exactly two elements",
                )
                .into());
            }
            let key = item
                .get_item(0)?
                .cast_into::<PyString>()
                .map_err(|_| PyTypeError::new_err("metadata keys must be strings"))?
                .to_str()?
                .to_owned();
            let value = item.get_item(1)?;
            let value = if value.is_instance_of::<PyBool>() {
                MetaVal::Bool(value.extract::<bool>()?)
            } else if value.is_instance_of::<PyString>() {
                MetaVal::try_from(value.extract::<String>()?)?
            } else {
                return Err(
                    PyTypeError::new_err("metadata values must be strings or booleans").into(),
                );
            };
            entries.push((Text::new(key)?, value));
        }
        Ok(Meta::try_from_iter(entries)?.into())
    }
}

impl PyMappingBackend for PyMeta {
    fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    fn iter_keys(&self) -> PyIterator {
        py_iterator!(
            self.0.clone(),
            |source| source.as_ref().iter(),
            |py, entry| {
                let (key, _) = entry;
                Ok(key.as_str().into_bound_py_any(py)?.unbind())
            },
        )
    }

    fn lookup(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Option<Py<PyAny>>> {
        // Metadata keys are ASCII. Strings that cannot be extracted as UTF-8
        // cannot match a stored key and have the same missing-key behavior.
        let Ok(key) = key.extract::<&str>() else {
            return Ok(None);
        };
        self.0
            .as_ref()
            .get(key)
            .map(|value| match value {
                MetaVal::String(value) => Ok(value.as_str().into_bound_py_any(py)?.unbind()),
                MetaVal::Bool(value) => Ok((*value).into_bound_py_any(py)?.unbind()),
            })
            .transpose()
    }

    fn contains(&self, key: &Bound<'_, PyAny>) -> bool {
        key.extract::<&str>()
            .is_ok_and(|key| self.0.as_ref().contains_key(key))
    }
}

py_mapping_methods!(PyMeta);

impl PyMeta {
    /// Converts optional Python input to owned metadata, using empty metadata for None.
    pub fn option_into_meta(value: Option<Self>) -> Meta {
        value.map_or_else(Meta::default, |value| value.0.as_ref().clone())
    }
}
