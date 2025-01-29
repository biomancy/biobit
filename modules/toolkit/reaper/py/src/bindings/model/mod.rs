use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use rna_pileup::PyRNAPileup;

mod rna_pileup;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "model";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyRNAPileup>()?;
    PyRNAPileup::type_object(parent.py()).setattr("__module__", &path)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
