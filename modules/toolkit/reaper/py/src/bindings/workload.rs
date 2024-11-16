use derive_getters::Dissolve;
use pyo3::prelude::*;

use biobit_reaper_rs::{Config, Workload};

use crate::cmp::PyEnrichment;
use crate::model::PyRNAPileup;
use crate::pcalling::PyByCutoff;
use crate::postfilter::PyNMS;

#[pyclass(eq, name = "Config")]
#[derive(Clone, PartialEq, Debug, Dissolve)]
pub struct PyConfig {
    pub model: PyRNAPileup,
    pub cmp: PyEnrichment,
    pub pcalling: PyByCutoff,
    pub postfilter: PyNMS,
}

#[pymethods]
impl PyConfig {
    #[new]
    fn new(model: PyRNAPileup, cmp: PyEnrichment, pcalling: PyByCutoff, postfilter: PyNMS) -> Self {
        Self {
            model,
            cmp,
            pcalling,
            postfilter,
        }
    }

    fn __getnewargs__(&self) -> (PyRNAPileup, PyEnrichment, PyByCutoff, PyNMS) {
        (
            self.model.clone(),
            self.cmp.clone(),
            self.pcalling.clone(),
            self.postfilter.clone(),
        )
    }
}

#[pyclass(eq, name = "Workload")]
#[derive(Clone, PartialEq, Debug, Dissolve)]
pub struct PyWorkload {
    pub regions: Vec<(String, usize, usize, PyConfig)>,
}

#[pymethods]
impl PyWorkload {
    #[new]
    fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    fn add_region(
        mut slf: PyRefMut<Self>,
        contig: String,
        start: usize,
        end: usize,
        config: PyConfig,
    ) -> PyRefMut<Self> {
        slf.regions.push((contig, start, end, config));
        slf
    }

    fn add_regions(
        mut slf: PyRefMut<Self>,
        regions: Vec<(String, usize, usize)>,
        config: PyConfig,
    ) -> PyRefMut<Self> {
        for (contig, start, end) in regions {
            slf.regions.push((contig, start, end, config.clone()));
        }
        slf
    }

    fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        self.regions.clone().into_pyobject(py).map(|x| x.unbind())
    }

    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        self.regions = state.extract(py)?;
        Ok(())
    }
}

impl From<PyWorkload> for Workload<String, usize, f32> {
    fn from(val: PyWorkload) -> Self {
        let mut workload = Workload::new();
        workload.regions = val
            .regions
            .into_iter()
            .map(|(contig, start, end, config)| {
                let config = Config::new(
                    config.model.into(),
                    config.cmp.into(),
                    config.pcalling.into(),
                    config.postfilter.into(),
                );
                (contig, start, end, config)
            })
            .collect();
        workload
    }
}
