use pyo3::prelude::*;

pub use biobit_io_rs::bam::{
    strdeductor, transform, AlignmentSegments, Reader, ReaderBuilder, SegmentedAlignment,
};
pub use reader::{IntoPyReader, PyReader};

mod reader;
pub mod utils;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "bam";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyReader>()?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
