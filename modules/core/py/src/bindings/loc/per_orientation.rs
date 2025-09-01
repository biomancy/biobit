use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};

use biobit_core_rs::loc::PerOrientation;

use crate::loc::IntoPyOrientation;
use crate::utils::type_hint_class_getitem;

#[pyclass(name = "PerOrientation")]
#[derive(Debug, Clone, From, Into)]
pub struct PyPerOrientation {
    internal: PerOrientation<Py<PyAny>>,
}

#[pymethods]
impl PyPerOrientation {
    #[new]
    #[pyo3(signature = (/, forward, reverse, dual))]
    pub fn new(forward: Py<PyAny>, reverse: Py<PyAny>, dual: Py<PyAny>) -> Self {
        PyPerOrientation {
            internal: PerOrientation {
                forward,
                reverse,
                dual,
            },
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

    #[getter]
    pub fn dual(&self) -> Py<PyAny> {
        self.internal.dual.clone()
    }

    #[setter]
    pub fn set_dual(&mut self, value: Py<PyAny>) {
        self.internal.dual = value;
    }

    pub fn get(&self, orientation: IntoPyOrientation) -> Py<PyAny> {
        let orientation = orientation.dissolve().0;
        self.internal.get(orientation).clone()
    }

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: Py<PyAny>) -> PyResult<Py<PyAny>> {
        type_hint_class_getitem(cls, args)
    }

    pub fn __getitem__(&self, orientation: IntoPyOrientation) -> Py<PyAny> {
        self.get(orientation)
    }

    pub fn __setitem__(&mut self, orientation: IntoPyOrientation, value: Py<PyAny>) {
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

    pub fn __getnewargs__(&self) -> (Py<PyAny>, Py<PyAny>, Py<PyAny>) {
        (
            self.internal.forward.clone(),
            self.internal.reverse.clone(),
            self.internal.dual.clone(),
        )
    }
}
