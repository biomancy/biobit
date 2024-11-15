use biobit_core_py::loc::{IntoPyChainInterval, IntoPyOrientation, Orientation, PyChainInterval};
use biobit_reaper_rs::model::{ControlModel, RNAPileup};
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use eyre::OptionExt;
use itertools::Itertools;
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

    fn __getstate__(
        &self,
    ) -> (
        f32,
        f32,
        f32,
        Vec<(Vec<PyChainInterval>, bool, Vec<usize>)>,
        Vec<(Vec<PyChainInterval>, bool, Vec<usize>)>,
        Vec<(Vec<PyChainInterval>, bool, Vec<usize>)>,
    ) {
        let modeling = self.rs.modeling().clone();
        let result = modeling
            .map(|_, m| {
                m.into_iter()
                    .map(|x| {
                        let x = x.dissolve();
                        let regions =
                            x.0.into_iter()
                                .map(|x| x.cast::<i64>().map(PyChainInterval::from))
                                .collect::<Option<Vec<_>>>()
                                .unwrap();
                        (regions, x.1, x.2)
                    })
                    .collect_vec()
            })
            .dissolve();
        (
            *self.rs.sensitivity(),
            *self.rs.control_baseline(),
            *self.rs.min_signal(),
            result.0,
            result.1,
            result.2,
        )
    }

    fn __setstate__(
        &mut self,
        state: (
            f32,
            f32,
            f32,
            Vec<(Vec<PyChainInterval>, bool, Vec<usize>)>,
            Vec<(Vec<PyChainInterval>, bool, Vec<usize>)>,
            Vec<(Vec<PyChainInterval>, bool, Vec<usize>)>,
        ),
    ) -> PyResult<()> {
        self.rs.set_sensitivity(state.0);
        self.rs.set_control_baseline(state.1);
        self.rs.set_min_signal(state.2);

        for (orientation, models) in [
            (Orientation::Forward, state.3),
            (Orientation::Reverse, state.4),
            (Orientation::Dual, state.5),
        ] {
            for (chains, uniform, winsizes) in models {
                let chains: Option<Vec<_>> =
                    chains.into_iter().map(|x| x.rs.cast::<usize>()).collect();

                let models = ControlModel::new(
                    chains.ok_or_eyre("Failed to cast ChainInterval to usize")?,
                    uniform,
                    winsizes,
                )?;
                self.rs.add_control_model(orientation, models);
            }
        }
        Ok(())
    }
}
