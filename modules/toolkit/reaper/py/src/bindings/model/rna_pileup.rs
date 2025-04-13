use biobit_core_py::loc::{IntoPyChainInterval, IntoPyOrientation};
use biobit_core_py::pickle;
use biobit_reaper_rs::model::{ControlModel, RNAPileup};
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::{OptionExt, Result};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::{pyclass, pymethods, PyRefMut, PyResult};

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
        slf.rs.add_control_model(orientation.0 .0, model);
        Ok(slf)
    }

    fn __getstate__(&self) -> Vec<u8> {
        pickle::to_bytes(&self.rs)
    }

    fn __setstate__(&mut self, state: Bound<PyBytes>) -> Result<()> {
        pickle::from_bytes(state.as_bytes()).map(|rs| self.rs = rs)
    }
}
