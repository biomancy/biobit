use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use by_cutoff::PyByCutoff;

mod by_cutoff;

pub fn register<'a>(
    parent: &Bound<'a, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'a, PyModule>> {
    let name = format!("{}.pcalling", parent.name()?);
    let module = PyModule::new(parent.py(), &name)?;

    module.add_class::<PyByCutoff>()?;
    PyByCutoff::type_object(parent.py()).setattr("__module__", &name)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
