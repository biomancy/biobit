use pyo3::prelude::*;

pub mod bam;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let io = PyModule::new_bound(parent.py(), &name)?;

    bam::register("bam", &io, sysmod)?;

    parent.add_submodule(&io)?;
    sysmod.set_item(io.name()?, &io)?;

    Ok(io)
}
