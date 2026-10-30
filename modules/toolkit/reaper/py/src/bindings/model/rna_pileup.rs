use biobit_core_py::loc::{IntoPyChainInterval, IntoPyOrientation};
use biobit_core_py::pickle;
use biobit_reaper_rs::model::{ControlModel, RNAPileup};
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::{OptionExt, Result};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::{PyRefMut, PyResult, pyclass, pymethods};

#[pyclass(from_py_object, eq, name = "RNAPileup")]
#[derive(Clone, PartialEq, Debug, Constructor, Dissolve, From, Into)]
// derive_more::Constructor generates explicit field names.
#[allow(clippy::redundant_field_names)]
pub struct PyRNAPileup {
    rs: RNAPileup<usize, f32>,
}

#[pymethods]
impl PyRNAPileup {
    #[new]
    fn __new__() -> Self {
        PyRNAPileup::new(RNAPileup::new())
    }

    fn set_scaling(mut slf: PyRefMut<Self>, signal: f32, control: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_scaling(signal, control)?;
        Ok(slf)
    }

    /// Sets the absolute numerical tolerance (finite and positive; default: 1e-6).
    /// It is used unchanged, independently of signal and control scaling.
    ///
    /// Raw coverage values differing by at most sensitivity are treated as
    /// identical during RLE encoding. Control modeling skips chains whose summed raw
    /// per-base coverage within the query is below sensitivity. Reaper also
    /// uses the same tolerance for RLE encoding of enrichment ratios.
    ///
    /// This is separate from the min_signal coverage threshold.
    fn set_sensitivity(mut slf: PyRefMut<Self>, sensitivity: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_sensitivity(sensitivity)?;
        Ok(slf)
    }

    fn set_control_baseline(
        mut slf: PyRefMut<Self>,
        control_baseline: f32,
    ) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_control_baseline(control_baseline)?;
        Ok(slf)
    }

    fn set_min_signal(mut slf: PyRefMut<Self>, min_signal: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_min_signal(min_signal)?;
        Ok(slf)
    }

    fn add_control_model(
        mut slf: PyRefMut<Self>,
        orientation: IntoPyOrientation,
        regions: Vec<IntoPyChainInterval>,
        uniform_baseline: bool,
        winsizes: Vec<usize>,
    ) -> PyResult<PyRefMut<Self>> {
        let regions = regions
            .into_iter()
            .map(|r| r.py.rs.cast::<usize>())
            .collect::<Option<Vec<_>>>()
            .ok_or_eyre("Failed to cast regions to usize")?;
        let model = ControlModel::new(regions, uniform_baseline, winsizes)?;
        slf.rs.add_control_model(orientation.0.0, model);
        Ok(slf)
    }

    fn __getstate__(&self) -> Vec<u8> {
        pickle::to_bytes(&self.rs)
    }

    fn __setstate__(&mut self, state: Bound<PyBytes>) -> Result<()> {
        pickle::from_bytes(state.as_bytes()).map(|rs| self.rs = rs)
    }
}
