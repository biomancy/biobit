use biobit_core_py::pickle;
use biobit_reaper_rs::cmp::Enrichment;
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::Result;
use pyo3::{PyRefMut, PyResult, pyclass, pymethods};

#[pyclass(from_py_object, eq, name = "Enrichment")]
#[derive(Clone, Debug, PartialEq, Constructor, Dissolve, From, Into)]
// derive_more::Constructor generates explicit field names.
#[allow(clippy::redundant_field_names)]
pub struct PyEnrichment {
    rs: Enrichment<f32>,
}

#[pymethods]
impl PyEnrichment {
    #[new]
    fn __new__() -> Self {
        PyEnrichment::new(Enrichment::new())
    }

    /// Adds a finite, non-negative pseudocount to both scaled profiles (default: 1).
    /// Positions without modeled signal retain zero enrichment.
    fn set_pseudocount(mut slf: PyRefMut<Self>, pseudocount: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_pseudocount(pseudocount)?;
        Ok(slf)
    }

    fn __getstate__(&self) -> Vec<u8> {
        pickle::to_bytes(&self.rs)
    }

    fn __setstate__(&mut self, state: Vec<u8>) -> Result<()> {
        pickle::from_bytes(&state).map(|rs| self.rs = rs)
    }
}
