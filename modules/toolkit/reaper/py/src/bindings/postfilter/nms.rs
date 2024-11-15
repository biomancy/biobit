use biobit_core_py::loc::{IntoPyChainInterval, IntoPyOrientation, Orientation, PyChainInterval};
use biobit_reaper_rs::postfilter::{NMSRegions, NMS};
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::OptionExt;
use itertools::Itertools;
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

    #[allow(clippy::type_complexity)]
    pub fn __getstate__(
        &self,
    ) -> (
        f32,
        usize,
        f32,
        f32,
        (usize, usize),
        (
            Vec<(bool, Vec<PyChainInterval>)>,
            Vec<(bool, Vec<PyChainInterval>)>,
            Vec<(bool, Vec<PyChainInterval>)>,
        ),
    ) {
        let rois = self.rs.roi().clone();
        let rois = rois
            .map(|_, y| {
                y.into_iter()
                    .map(|x| {
                        (
                            x.uniform_baseline,
                            x.regions
                                .into_iter()
                                .map(|x| PyChainInterval::from(x.cast::<i64>().unwrap()))
                                .collect_vec(),
                        )
                    })
                    .collect_vec()
            })
            .dissolve();

        (
            *self.rs.fecutoff(),
            *self.rs.group_within(),
            *self.rs.slopfrac(),
            *self.rs.sensitivity(),
            *self.rs.sloplim(),
            rois,
        )
    }

    #[allow(clippy::type_complexity)]
    pub fn __setstate__(
        &mut self,
        state: (
            f32,
            usize,
            f32,
            f32,
            (usize, usize),
            (
                Vec<(bool, Vec<PyChainInterval>)>,
                Vec<(bool, Vec<PyChainInterval>)>,
                Vec<(bool, Vec<PyChainInterval>)>,
            ),
        ),
    ) -> PyResult<()> {
        self.rs.set_fecutoff(state.0).unwrap();
        self.rs.set_group_within(state.1).unwrap();
        self.rs.set_slopfrac(state.2).unwrap();
        self.rs.set_sensitivity(state.3).unwrap();
        self.rs.set_sloplim(state.4 .0, state.4 .1).unwrap();

        for (orientation, regions) in [
            (Orientation::Forward, state.5 .0),
            (Orientation::Reverse, state.5 .1),
            (Orientation::Dual, state.5 .2),
        ] {
            for (uniform, chains) in regions {
                let chains: Option<Vec<_>> =
                    chains.into_iter().map(|x| x.rs.cast::<usize>()).collect();
                let regions = NMSRegions::new(
                    chains.ok_or_eyre("Failed to cast ChainInterval to usize")?,
                    uniform,
                )?;
                self.rs.add_regions(orientation, regions);
            }
        }
        Ok(())
    }
}
