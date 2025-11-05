use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};

use biobit_core_rs::loc::PerStrand;

use crate::loc::IntoPyStrand;
use crate::utils::type_hint_class_getitem;

#[pyclass(name = "PerStrand")]
#[derive(Debug, Clone, From, Into)]
pub struct PyPerStrand {
    internal: PerStrand<Py<PyAny>>,
}

#[pymethods]
impl PyPerStrand {
    #[new]
    #[pyo3(signature = (/, forward, reverse))]
    pub fn new(forward: Py<PyAny>, reverse: Py<PyAny>) -> Self {
        PyPerStrand {
            internal: PerStrand { forward, reverse },
        }
    }

    #[getter]
    pub fn forward(&self) -> Py<PyAny> {
        self.internal.forward.clone()
    }

    #[setter]
    pub fn set_forward(&mut self, value: Py<PyAny>) {
        self.internal.forward = value;
    }

    #[getter]
    pub fn reverse(&self) -> Py<PyAny> {
        self.internal.reverse.clone()
    }

    #[setter]
    pub fn set_reverse(&mut self, value: Py<PyAny>) {
        self.internal.reverse = value;
    }

    pub fn get(&self, strand: IntoPyStrand) -> Py<PyAny> {
        let strand = strand.dissolve().0;
        self.internal.get(strand).clone()
    }

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: Py<PyAny>) -> PyResult<Py<PyAny>> {
        type_hint_class_getitem(cls, args)
    }

    pub fn __getitem__(&self, strand: IntoPyStrand) -> Py<PyAny> {
        self.get(strand)
    }

    pub fn __setitem__(&mut self, strand: IntoPyStrand, value: Py<PyAny>) {
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

    pub fn __getnewargs__(&self) -> (Py<PyAny>, Py<PyAny>) {
        (self.internal.forward.clone(), self.internal.reverse.clone())
    }
}
