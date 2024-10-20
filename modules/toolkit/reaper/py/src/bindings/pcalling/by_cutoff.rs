use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use pyo3::{pyclass, pymethods, PyRefMut};

use biobit_reaper_rs::pcalling::ByCutoff;

#[pyclass(eq, name = "ByCutoff")]
#[derive(Clone, PartialEq, Debug, Constructor, Dissolve, From, Into)]
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

    fn __getstate__(&self) -> (usize, usize, f32) {
        (
            *self.rs.min_length(),
            *self.rs.merge_within(),
            *self.rs.cutoff(),
        )
    }

    fn __setstate__(&mut self, state: (usize, usize, f32)) {
        self.rs.set_min_length(state.0);
        self.rs.set_merge_within(state.1);
        self.rs.set_cutoff(state.2);
    }
}
