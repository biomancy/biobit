use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

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

    pub fn __getitem__(&self, strand: IntoPyStrand) -> PyObject {
        self.get(strand)
    }

    pub fn __setitem__(&mut self, strand: IntoPyStrand, value: PyObject) {
        let strand = strand.dissolve().0;
        self.internal.get_mut(strand).clone_from(&value);
    }

    pub fn __hash__(&self, py: Python) -> PyResult<isize> {
        PyTuple::new_bound(
            py,
            &[self.internal.forward.clone(), self.internal.reverse.clone()],
        )
        .hash()
    }

    pub fn __richcmp__(&self, py: Python, other: &PyPerStrand, op: CompareOp) -> PyResult<bool> {
        let slf = PyTuple::new_bound(
            py,
            &[self.internal.forward.clone(), self.internal.reverse.clone()],
        );
        let other = PyTuple::new_bound(
            py,
            &[
                other.internal.forward.clone(),
                other.internal.reverse.clone(),
            ],
        );

        Ok(slf.rich_compare(other, op)?.extract()?)
    }

    pub fn __getnewargs__(&self) -> (PyObject, PyObject) {
        (self.internal.forward.clone(), self.internal.reverse.clone())
    }
}
