use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

mod builder;
mod engine;
mod resolution;

pub use builder::{EngineBuilder, PyEngineBuilder};
pub use engine::{Engine, PyEngine};

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyEngine>()?
        .add_class::<PyEngineBuilder>()?
        .add_submodule(&resolution::construct(py, &format!("{name}.resolution"))?)?
        .finish();

    Ok(module)
}
