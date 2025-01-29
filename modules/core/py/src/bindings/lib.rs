use pyo3::prelude::*;

pub use biobit_core_rs::{num, parallelism, source, LendingIterator};

pub mod fallible_py_runtime;
pub mod loc;
pub mod ngs;
pub mod pickle;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "core";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    loc::register(&path, &module, sysmod)?;
    ngs::register(&path, &module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
