use pyo3::prelude::*;

pub mod bam;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "io";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    bam::register(&path, &module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
