mod ripper;
mod result;
mod config;

use pyo3::prelude::*;

// pub use countit::PyCountIt;
// pub use result::{PyCounts, PyStats};


pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let ripper = PyModule::new_bound(parent.py(), &name)?;

    // ripper.add_class::<PyCountIt>()?;
    // ripper.add_class::<PyCounts>()?;
    // ripper.add_class::<PyStats>()?;

    parent.add_submodule(&ripper)?;
    sysmod.set_item(ripper.name()?, &ripper)?;

    Ok(ripper)
}
