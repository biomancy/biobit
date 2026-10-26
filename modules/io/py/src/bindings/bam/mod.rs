use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use biobit_io_rs::bam::{
    AlignmentSegments, Reader, ReaderBuilder, SegmentedAlignment, strdeductor, transform,
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
    module.add(
        "IntoReader",
        py.get_type::<PyReader>()
            .bitor(py.get_type::<pyo3::types::PyString>())?,
    )?;

    module.add("__all__", ["IntoReader", "Reader"])?;

    Ok(module)
}
