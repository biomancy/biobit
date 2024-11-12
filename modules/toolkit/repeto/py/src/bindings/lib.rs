use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyResult};

pub mod optimize;
pub mod predict;
pub mod repeats;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.repeto", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    repeats::register(&module, sysmod)?;
    optimize::register(&module, sysmod)?;
    predict::register(&module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
