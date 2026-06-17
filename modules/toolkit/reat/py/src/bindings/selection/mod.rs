use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub use mismatches::PyMismatches;
pub use required_or_mismatches::PyRequiredOrMismatches;
pub use required_sites::PyRequiredSites;
pub use selector::IntoPySelector;

mod mismatches;
mod required_or_mismatches;
mod required_sites;
mod selector;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyMismatches>()?
        .add_class::<PyRequiredSites>()?
        .add_class::<PyRequiredOrMismatches>()?
        .finish();

    Ok(module)
}
