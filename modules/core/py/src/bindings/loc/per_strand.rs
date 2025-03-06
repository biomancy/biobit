use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use std::ffi::CString;

use biobit_core_rs::loc::PerStrand;

use crate::loc::IntoPyStrand;

#[pyclass(name = "PerStrand")]
#[derive(Debug, Clone, From, Into)]
pub struct PyPerStrand {
    internal: PerStrand<PyObject>,
}

#[pymethods]
impl PyPerStrand {
    #[new]
    #[pyo3(signature = (/, forward, reverse))]
    pub fn new(forward: PyObject, reverse: PyObject) -> Self {
        PyPerStrand {
            internal: PerStrand { forward, reverse },
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

    pub fn get(&self, strand: IntoPyStrand) -> PyObject {
        let strand = strand.dissolve().0;
        self.internal.get(strand).clone()
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

    pub fn __getitem__(&self, strand: IntoPyStrand) -> PyObject {
        self.get(strand)
    }

    pub fn __setitem__(&mut self, strand: IntoPyStrand, value: PyObject) {
        let strand = strand.dissolve().0;
        self.internal.get_mut(strand).clone_from(&value);
    }

    pub fn __hash__(&self, py: Python) -> PyResult<isize> {
        PyTuple::new(
            py,
            &[self.internal.forward.clone(), self.internal.reverse.clone()],
        )?
        .hash()
    }

    pub fn __richcmp__(&self, py: Python, other: &PyPerStrand, op: CompareOp) -> PyResult<bool> {
        let slf = PyTuple::new(
            py,
            &[self.internal.forward.clone(), self.internal.reverse.clone()],
        )?;
        let other = PyTuple::new(
            py,
            &[
                other.internal.forward.clone(),
                other.internal.reverse.clone(),
            ],
        )?;

        slf.rich_compare(other, op)?.extract()
    }

    pub fn __getnewargs__(&self) -> (PyObject, PyObject) {
        (self.internal.forward.clone(), self.internal.reverse.clone())
    }
}
