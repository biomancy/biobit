use pyo3::prelude::*;

pub use biobit_core_rs::{LendingIterator, parallelism, source, num};

pub mod loc;
pub mod ngs;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let core = PyModule::new_bound(parent.py(), &name)?;

    loc::register("loc", &core, sysmod)?;
    ngs::register("ngs", &core, sysmod)?;

    parent.add_submodule(&core)?;
    sysmod.set_item(core.name()?, &core)?;

    Ok(core)
}
