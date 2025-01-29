use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyResult};

pub mod optimize;
pub mod predict;
pub mod repeats;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "repeto";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    repeats::register(&path, &module, sysmod)?;
    optimize::register(&path, &module, sysmod)?;
    predict::register(&path, &module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
