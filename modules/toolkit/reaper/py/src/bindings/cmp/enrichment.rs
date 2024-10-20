use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use pyo3::{pyclass, pymethods, PyRefMut};

use biobit_reaper_rs::cmp::Enrichment;

#[pyclass(eq, name = "Enrichment")]
#[derive(Clone, Debug, PartialEq, Constructor, Dissolve, From, Into)]
pub struct PyEnrichment {
    rs: Enrichment<f32>,
}

#[pymethods]
impl PyEnrichment {
    #[new]
    fn __new__() -> Self {
        PyEnrichment::new(Enrichment::new())
    }

    fn set_scaling(mut slf: PyRefMut<Self>, signal: f32, control: f32) -> PyRefMut<Self> {
        slf.rs.set_scaling(signal, control);
        slf
    }

    fn __getstate__(&self) -> (f32, f32) {
        (self.rs.scaling.signal, self.rs.scaling.control)
    }

    fn __setstate__(&mut self, state: (f32, f32)) {
        self.rs.set_scaling(state.0, state.1);
    }
}
