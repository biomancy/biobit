use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use by_cutoff::PyByCutoff;
use pyo3::prelude::*;

mod by_cutoff;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyByCutoff>()?
        .finish();

    Ok(module)
}
