use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use by_cutoff::PyByCutoff;

mod by_cutoff;

pub fn register<'a>(
    parent: &Bound<'a, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'a, PyModule>> {
    let name = format!("{}.pcalling", parent.name()?);
    let cmp = PyModule::new_bound(parent.py(), &name)?;

    cmp.add_class::<PyByCutoff>()?;
    PyByCutoff::type_object_bound(parent.py()).setattr("__module__", &name)?;

    parent.add_submodule(&cmp)?;
    sysmod.set_item(cmp.name()?, &cmp)?;

    Ok(cmp)
}
