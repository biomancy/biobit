use pyo3::prelude::*;

mod builder;
mod engine;
mod resolution;

pub use builder::{EngineBuilder, PyEngineBuilder};
pub use engine::{Engine, PyEngine};

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "rigid";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyEngine>()?;
    module.add_class::<PyEngineBuilder>()?;

    resolution::register(&path, &module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
