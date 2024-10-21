use pyo3::prelude::*;

pub use result::{
    Counts, PartitionMetrics, PyCounts, PyPartitionMetrics, PyResolutionOutcome, ResolutionOutcomes,
};

mod result;
pub mod rigid;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.countit", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyResolutionOutcome>()?;
    module.add_class::<PyCounts>()?;
    module.add_class::<PyPartitionMetrics>()?;

    rigid::register(&module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
