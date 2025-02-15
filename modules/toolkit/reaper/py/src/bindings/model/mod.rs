use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
pub use rna_pileup::PyRNAPileup;

mod rna_pileup;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyRNAPileup>()?
        .finish();

    Ok(module)
}
