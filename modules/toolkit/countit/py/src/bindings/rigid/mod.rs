use pyo3::prelude::*;

mod builder;
mod engine;
mod resolution;

pub use builder::{EngineBuilder, PyEngineBuilder};
pub use engine::{Engine, PyEngine};

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.rigid", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyEngine>()?;
    module.add_class::<PyEngineBuilder>()?;

    resolution::register(&module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
