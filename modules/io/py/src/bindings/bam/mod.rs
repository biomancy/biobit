use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use biobit_io_rs::bam::{
    strdeductor, transform, AlignmentSegments, Reader, ReaderBuilder, SegmentedAlignment,
};
use pyo3::prelude::*;
pub use reader::{IntoPyReader, PyReader};

mod reader;
pub mod utils;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyReader>()?
        .finish();

    Ok(module)
}
