use crate::interval_tree::overlap;
pub use biobit_collections_rs::interval_tree::{Bits, BitsBuilder};
use biobit_collections_rs::interval_tree::{Builder, ITree};
use biobit_core_py::loc::IntoPyInterval;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::prelude::*;

#[pyclass(name = "BitsBuilder")]
#[repr(transparent)]
#[derive(Default, Dissolve, From, Into)]
pub struct PyBitsBuilder(pub BitsBuilder<i64, PyObject>);

#[pymethods]
impl PyBitsBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        data: Vec<(IntoPyInterval, PyObject)>,
    ) -> PyRefMut<'py, Self> {
        let mut builder = std::mem::take(&mut slf.0);
        for (interval, element) in data {
            builder = builder.addi(&interval.0.bind(py).borrow().rs, element);
        }
        slf.0 = builder;
        slf
    }

    pub fn addi<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        interval: IntoPyInterval,
        element: PyObject,
    ) -> PyRefMut<'py, Self> {
        slf.0 = std::mem::take(&mut slf.0).addi(&interval.0.bind(py).borrow().rs, element);
        slf
    }

    pub fn build(&mut self) -> PyBits {
        PyBits(std::mem::take(&mut self.0).build())
    }
}

#[pyclass(name = "Bits")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyBits(pub Bits<i64, PyObject>);

#[pymethods]
impl PyBits {
    #[staticmethod]
    pub fn builder() -> PyBitsBuilder {
        PyBitsBuilder::new()
    }

    pub fn overlap<'py>(
        &self,
        py: Python<'py>,
        intervals: Vec<IntoPyInterval>,
        mut buffer: PyRefMut<'py, overlap::PyElements>,
    ) -> PyRefMut<'py, overlap::PyElements> {
        let intervals = intervals
            .iter()
            .map(|x| x.0.bind(py).borrow().rs)
            .collect::<Vec<_>>();

        self.0.overlap_single_element(&intervals, &mut buffer.0);
        buffer
    }
}
