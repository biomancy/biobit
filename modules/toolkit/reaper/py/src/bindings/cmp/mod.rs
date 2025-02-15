use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use enrichment::PyEnrichment;
use pyo3::prelude::*;

mod enrichment;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyEnrichment>()?
        .finish();

    Ok(module)
}
