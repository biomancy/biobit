use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
pub use result::{
    Counts, PartitionMetrics, PyCounts, PyPartitionMetrics, PyResolutionOutcome, ResolutionOutcomes,
};

mod result;
pub mod rigid;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&rigid::construct(py, &format!("{name}.rigid"))?)?
        .add_class::<PyResolutionOutcome>()?
        .add_class::<PyCounts>()?
        .add_class::<PyPartitionMetrics>()?
        .finish();

    Ok(module)
}
