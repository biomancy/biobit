use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use pyo3::{pyclass, pymethods, PyRefMut};

use biobit_reaper_rs::model::RNAPileup;

#[pyclass(eq, name = "RNAPileup")]
#[derive(Clone, PartialEq, Debug, Constructor, Dissolve, From, Into)]
pub struct PyRNAPileup {
    rs: RNAPileup<usize, f32>,
}

#[pymethods]
impl PyRNAPileup {
    #[new]
    fn __new__() -> Self {
        PyRNAPileup::new(RNAPileup::new())
    }

    fn set_sensitivity(mut slf: PyRefMut<Self>, sensitivity: f32) -> PyRefMut<Self> {
        slf.rs.set_sensitivity(sensitivity);
        slf
    }

    fn set_control_baseline(mut slf: PyRefMut<Self>, control_baseline: f32) -> PyRefMut<Self> {
        slf.rs.set_control_baseline(control_baseline);
        slf
    }

    fn set_min_signal(mut slf: PyRefMut<Self>, min_signal: f32) -> PyRefMut<Self> {
        slf.rs.set_min_signal(min_signal);
        slf
    }

    fn __getstate__(&self) -> (f32, f32, f32) {
        (
            *self.rs.sensitivity(),
            *self.rs.control_baseline(),
            *self.rs.min_signal(),
        )
    }

    fn __setstate__(&mut self, state: (f32, f32, f32)) {
        self.rs.set_sensitivity(state.0);
        self.rs.set_control_baseline(state.1);
        self.rs.set_min_signal(state.2);
    }
}
