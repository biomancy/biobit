use biobit_core_py::pickle;
use biobit_reaper_rs::pcalling::ByCutoff;
use bitcode::{Decode, Encode};
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::Result;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::{Bound, PyRefMut, pyclass, pymethods};

#[pyclass(eq, name = "ByCutoff")]
#[derive(Encode, Decode, Clone, PartialEq, Debug, Constructor, Dissolve, From, Into)]
pub struct PyByCutoff {
    rs: ByCutoff<usize, f32>,
}

#[pymethods]
impl PyByCutoff {
    #[new]
    fn __new__() -> Self {
        PyByCutoff::new(ByCutoff::new())
    }

    fn set_min_length(mut slf: PyRefMut<Self>, min_length: usize) -> PyRefMut<Self> {
        slf.rs.set_min_length(min_length);
        slf
    }

    fn set_merge_within(mut slf: PyRefMut<Self>, merge_within: usize) -> PyRefMut<Self> {
        slf.rs.set_merge_within(merge_within);
        slf
    }

    fn set_cutoff(mut slf: PyRefMut<Self>, cutoff: f32) -> PyRefMut<Self> {
        slf.rs.set_cutoff(cutoff);
        slf
    }

    fn __getstate__(&self) -> Vec<u8> {
        pickle::to_bytes(&self.rs)
    }

    fn __setstate__(&mut self, state: Bound<PyBytes>) -> Result<()> {
        pickle::from_bytes(state.as_bytes()).map(|rs| self.rs = rs)
    }
}
