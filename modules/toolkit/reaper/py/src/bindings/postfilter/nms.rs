use biobit_core_py::loc::{IntoPyChainInterval, IntoPyOrientation};
use biobit_core_py::pickle;
use biobit_reaper_rs::postfilter::{NMSRegions, NMS};
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::{OptionExt, Result};
use pyo3::prelude::*;

#[pyclass(eq, name = "NMS")]
#[derive(Clone, PartialEq, Debug, Constructor, Dissolve, From, Into)]
pub struct PyNMS {
    rs: NMS<usize, f32>,
}

#[pymethods]
impl PyNMS {
    #[new]
    pub fn __new__() -> Self {
        PyNMS::new(NMS::new())
    }

    pub fn set_fecutoff(mut slf: PyRefMut<Self>, fecutoff: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_fecutoff(fecutoff)?;
        Ok(slf)
    }

    pub fn set_group_within(
        mut slf: PyRefMut<Self>,
        group_within: usize,
    ) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_group_within(group_within)?;
        Ok(slf)
    }

    pub fn set_slopfrac(mut slf: PyRefMut<Self>, slopfrac: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_slopfrac(slopfrac)?;
        Ok(slf)
    }

    pub fn set_sloplim(
        mut slf: PyRefMut<Self>,
        minslop: usize,
        maxslop: usize,
    ) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_sloplim(minslop, maxslop)?;
        Ok(slf)
    }

    pub fn set_sensitivity(mut slf: PyRefMut<Self>, sensitivity: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_sensitivity(sensitivity)?;
        Ok(slf)
    }

    pub fn add_regions(
        mut slf: PyRefMut<Self>,
        orientation: IntoPyOrientation,
        uniform_baseline: bool,
        regions: Vec<IntoPyChainInterval>,
    ) -> PyResult<PyRefMut<Self>> {
        let regions: Option<_> = regions
            .into_iter()
            .map(|x| x.py.rs.cast::<usize>())
            .collect();
        let regions = NMSRegions::new(
            regions.ok_or_eyre("Failed to cast ChainInterval to usize")?,
            uniform_baseline,
        )?;

        slf.rs.add_regions(orientation.0 .0, regions);
        Ok(slf)
    }

    fn __getstate__(&self) -> Vec<u8> {
        pickle::to_bytes(&self.rs)
    }

    fn __setstate__(&mut self, state: Vec<u8>) -> Result<()> {
        pickle::from_bytes(&state).map(|rs| self.rs = rs)
    }
}
