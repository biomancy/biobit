use pyo3::prelude::*;

pub use biobit_core_rs::{num, parallelism, source, LendingIterator};

pub mod fallible_py_runtime;
pub mod loc;
pub mod ngs;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.core", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    loc::register(&module, sysmod)?;
    ngs::register(&module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
