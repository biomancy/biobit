use pyo3::{Bound, PyAny, PyResult};
use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};

pub mod optimize;
pub mod predict;
pub mod repeats;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.repeto", parent.name()?);
    let repeto = PyModule::new_bound(parent.py(), &name)?;

    repeats::register(&repeto, sysmod)?;
    optimize::register(&repeto, sysmod)?;
    predict::register(&repeto, sysmod)?;

    parent.add_submodule(&repeto)?;
    sysmod.set_item(repeto.name()?, &repeto)?;

    Ok(repeto)
}
