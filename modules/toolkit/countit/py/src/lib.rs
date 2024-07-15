use pyo3::prelude::*;

pub use countit::PyCountIt;
pub use result::{PyCounts, PyStats};

mod countit;
mod result;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let countit = PyModule::new_bound(parent.py(), &name)?;

    countit.add_class::<PyCountIt>()?;
    countit.add_class::<PyCounts>()?;
    countit.add_class::<PyStats>()?;

    parent.add_submodule(&countit)?;
    sysmod.set_item(countit.name()?, &countit)?;

    Ok(countit)
}
