use pyo3::prelude::*;

pub use countit::PyCountIt;
pub use result::{PyCounts, PyStats};

mod countit;
mod result;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.countit", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyCountIt>()?;
    module.add_class::<PyCounts>()?;
    module.add_class::<PyStats>()?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
