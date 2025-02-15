use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult, Python};

pub mod optimize;
pub mod predict;
pub mod repeats;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&repeats::construct(py, &format!("{name}.repeats"))?)?
        .add_submodule(&optimize::construct(py, &format!("{name}.optimize"))?)?
        .add_submodule(&predict::construct(py, &format!("{name}.predict"))?)?
        .finish();

    Ok(module)
}
