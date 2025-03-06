use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use std::ffi::CString;

use biobit_core_rs::loc::PerOrientation;

use crate::loc::IntoPyOrientation;

#[pyclass(name = "PerOrientation")]
#[derive(Debug, Clone, From, Into)]
pub struct PyPerOrientation {
    internal: PerOrientation<PyObject>,
}

#[pymethods]
impl PyPerOrientation {
    #[new]
    #[pyo3(signature = (/, forward, reverse, dual))]
    pub fn new(forward: PyObject, reverse: PyObject, dual: PyObject) -> Self {
        PyPerOrientation {
            internal: PerOrientation {
                forward,
                reverse,
                dual,
            },
        }
    }

    #[getter]
    pub fn forward(&self) -> PyObject {
        self.internal.forward.clone()
    }

    #[setter]
    pub fn set_forward(&mut self, value: PyObject) {
        self.internal.forward = value;
    }

    #[getter]
    pub fn reverse(&self) -> PyObject {
        self.internal.reverse.clone()
    }

    #[setter]
    pub fn set_reverse(&mut self, value: PyObject) {
        self.internal.reverse = value;
    }

    #[getter]
    pub fn dual(&self) -> PyObject {
        self.internal.dual.clone()
    }

    #[setter]
    pub fn set_dual(&mut self, value: PyObject) {
        self.internal.dual = value;
    }

    pub fn get(&self, orientation: IntoPyOrientation) -> PyObject {
        let orientation = orientation.dissolve().0;
        self.internal.get(orientation).clone()
    }

    #[classmethod]
    pub fn __class_getitem__(
        cls: &Bound<PyType>,
        args: PyObject,
        py: Python,
    ) -> PyResult<PyObject> {
        let locals = PyDict::new(py);
        locals.set_item("cls", cls)?;
        locals.set_item("args", args)?;

        py.run(
            &CString::new(r#"import types;result = types.GenericAlias(cls, args);"#)?,
            None,
            Some(&locals),
        )?;

        Ok(locals.get_item("result")?.unwrap().unbind())
    }

    pub fn __getitem__(&self, orientation: IntoPyOrientation) -> PyObject {
        self.get(orientation)
    }

    pub fn __setitem__(&mut self, orientation: IntoPyOrientation, value: PyObject) {
        let orientation = orientation.dissolve().0;
        self.internal.get_mut(orientation).clone_from(&value);
    }

    pub fn __hash__(&self, py: Python) -> PyResult<isize> {
        PyTuple::new(
            py,
            &[
                self.internal.forward.clone(),
                self.internal.reverse.clone(),
                self.internal.dual.clone(),
            ],
        )?
        .hash()
    }

    pub fn __richcmp__(
        &self,
        py: Python,
        other: &PyPerOrientation,
        op: CompareOp,
    ) -> PyResult<bool> {
        let slf = PyTuple::new(
            py,
            &[
                self.internal.forward.clone(),
                self.internal.reverse.clone(),
                self.internal.dual.clone(),
            ],
        )?;
        let other = PyTuple::new(
            py,
            &[
                other.internal.forward.clone(),
                other.internal.reverse.clone(),
                other.internal.dual.clone(),
            ],
        )?;

        slf.rich_compare(other, op)?.extract()
    }

    pub fn __getnewargs__(&self) -> (PyObject, PyObject, PyObject) {
        (
            self.internal.forward.clone(),
            self.internal.reverse.clone(),
            self.internal.dual.clone(),
        )
    }
}
