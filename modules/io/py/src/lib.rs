use pyo3::prelude::*;

pub mod bam;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.io", parent.name()?);
    let io = PyModule::new_bound(parent.py(), &name)?;

    bam::register(&io, sysmod)?;

    parent.add_submodule(&io)?;
    sysmod.set_item(io.name()?, &io)?;

    Ok(io)
}
