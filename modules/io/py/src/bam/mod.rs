use pyo3::prelude::*;

pub use biobit_io_rs::bam::{AlignmentSegments, Reader, strdeductor, transform, ReaderBuilder};
pub use reader::{IntoPyReader, PyReader};

mod reader;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let bam = PyModule::new_bound(parent.py(), &name)?;

    bam.add_class::<PyReader>()?;

    parent.add_submodule(&bam)?;
    sysmod.set_item(bam.name()?, &bam)?;

    Ok(bam)
}
