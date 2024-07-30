use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use pyo3::prelude::*;

use biobit_ripper_rs::config;

#[pyclass(name = "Config")]
#[derive(Clone, Debug, Dissolve, Default, From, Into, Constructor)]
pub struct PyConfig {
    rs: config::Config<usize, f64>,
}

#[pymethods]
impl PyConfig {
    #[new]
    fn __new__() -> Self {
        PyConfig::new(config::Config::default())
    }

    fn with_sensitivity(mut slf: PyRefMut<Self>, sensitivity: f64) -> PyRefMut<Self> {
        slf.rs.sensitivity = sensitivity;
        slf
    }

    fn with_control_baseline(mut slf: PyRefMut<Self>, control_baseline: f64) -> PyRefMut<Self> {
        slf.rs.control_baseline = control_baseline;
        slf
    }

    fn with_scaling_factors(mut slf: PyRefMut<Self>, signal: f64, control: f64) -> PyRefMut<Self> {
        slf.rs.signal_scaling = signal;
        slf.rs.control_scaling = control;
        slf
    }

    fn with_min_raw_signal(mut slf: PyRefMut<Self>, min_raw_signal: f64) -> PyRefMut<Self> {
        slf.rs.min_raw_signal = min_raw_signal;
        slf
    }

    fn with_pcalling_params(
        mut slf: PyRefMut<Self>,
        min_length: usize,
        merge_within: usize,
        cutoff: f64,
    ) -> PyRefMut<Self> {
        slf.rs.pcalling = config::PCalling::new(min_length, merge_within, cutoff);
        slf
    }
}
