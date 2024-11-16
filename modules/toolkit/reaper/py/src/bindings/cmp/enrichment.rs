use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use pyo3::{pyclass, pymethods, PyErr, PyRefMut, PyResult};

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

    fn __getstate__(&self) -> Vec<u8> {
        bitcode::encode(&self.rs)
    }

    fn __setstate__(&mut self, state: Vec<u8>) -> PyResult<()> {
        bitcode::decode(&state)
            .map(|rs| self.rs = rs)
            .map_err(|x| PyErr::from(eyre::Report::from(x)))
    }
}
