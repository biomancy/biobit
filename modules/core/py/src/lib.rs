use pyo3::prelude::*;

pub use biobit_core_rs::{LendingIterator, num, parallelism, source};

pub mod loc;
pub mod ngs;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.core", parent.name()?);
    let core = PyModule::new_bound(parent.py(), &name)?;

    loc::register(&core, sysmod)?;
    ngs::register(&core, sysmod)?;

    parent.add_submodule(&core)?;
    sysmod.set_item(core.name()?, &core)?;

    Ok(core)
}
