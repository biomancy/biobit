use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use enrichment::PyEnrichment;

mod enrichment;

pub fn register<'a>(
    parent: &Bound<'a, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'a, PyModule>> {
    let name = format!("{}.cmp", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyEnrichment>()?;
    PyEnrichment::type_object_bound(parent.py()).setattr("__module__", &name)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
