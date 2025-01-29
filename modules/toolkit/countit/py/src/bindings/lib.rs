use pyo3::prelude::*;

pub use result::{
    Counts, PartitionMetrics, PyCounts, PyPartitionMetrics, PyResolutionOutcome, ResolutionOutcomes,
};

mod result;
pub mod rigid;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "countit";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyResolutionOutcome>()?;
    module.add_class::<PyCounts>()?;
    module.add_class::<PyPartitionMetrics>()?;

    rigid::register(&path, &module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
