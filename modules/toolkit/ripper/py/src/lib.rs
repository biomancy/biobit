use pyo3::prelude::*;

pub use config::PyConfig;
pub use result::{PyPeak, PyRegion, PyRipped};
pub use ripper::PyRipper;

mod config;
mod result;
mod ripper;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let ripper = PyModule::new_bound(parent.py(), &name)?;

    ripper.add_class::<PyRipper>()?;
    ripper.add_class::<PyConfig>()?;
    ripper.add_class::<PyPeak>()?;
    ripper.add_class::<PyRegion>()?;
    ripper.add_class::<PyRipped>()?;

    parent.add_submodule(&ripper)?;
    sysmod.set_item(ripper.name()?, &ripper)?;

    Ok(ripper)
}
