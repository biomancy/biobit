use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::PathBuf;

use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};

#[pyclass(skip_from_py_object, frozen, eq, ord, hash, name = "FilePath")]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PyFilePath(pub Frozen<biobit_bioproj_rs::FilePath>);

frozen_wrapper!(PyFilePath, biobit_bioproj_rs::FilePath);

impl<'a, 'py> FromPyObject<'a, 'py> for PyFilePath {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = obj.cast::<Self>() {
            return Ok(Self(value.borrow().0.clone()));
        }
        let path = obj.extract::<PathBuf>()?;
        Self::from_absolute(path).map_err(Into::into)
    }
}

#[pymethods]
impl PyFilePath {
    #[staticmethod]
    fn from_absolute(value: PathBuf) -> PyBioprojResult<Self> {
        Ok(biobit_bioproj_rs::FilePath::from_absolute(value)?.into())
    }

    #[staticmethod]
    fn from_relative(value: PathBuf, root: PathBuf) -> PyBioprojResult<Self> {
        let root = biobit_bioproj_rs::Root::new(root)?;
        Ok(biobit_bioproj_rs::FilePath::from_relative(value, root)?.into())
    }

    #[getter]
    fn absolute(&self) -> PathBuf {
        self.0.as_ref().absolute()
    }

    fn __fspath__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.absolute().as_os_str().into_bound_py_any(py)?.unbind())
    }

    fn __str__(&self) -> String {
        self.0.as_ref().to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "FilePath({:?}, absolute={:?})",
            self.0.as_ref().as_path(),
            self.0.as_ref().absolute()
        )
    }
}
