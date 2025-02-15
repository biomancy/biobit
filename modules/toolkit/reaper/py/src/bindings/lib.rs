use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
pub use reaper::PyReaper;
pub use result::{PyHarvest, PyHarvestRegion, PyPeak};
pub use workload::{PyConfig, PyWorkload};

pub mod cmp;
pub mod model;
pub mod pcalling;
mod postfilter;
mod reaper;
mod result;
mod workload;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyReaper>()?
        .add_class::<PyConfig>()?
        .add_class::<PyWorkload>()?
        .add_class::<PyPeak>()?
        .add_class::<PyHarvest>()?
        .add_class::<PyHarvestRegion>()?
        .add_submodule(&cmp::construct(py, &format!("{name}.cmp"))?)?
        .add_submodule(&model::construct(py, &format!("{name}.model"))?)?
        .add_submodule(&pcalling::construct(py, &format!("{name}.pcalling"))?)?
        .add_submodule(&postfilter::construct(py, &format!("{name}.postfilter"))?)?
        .finish();

    Ok(module)
}
