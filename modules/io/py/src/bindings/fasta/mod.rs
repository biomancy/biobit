use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub use biobit_io_rs::fasta::DEFAULT_LINE_WIDTH;
pub use indexed_reader::{IndexedReader, IndexedReaderMutOp, PyIndexedReader};
pub use reader::{PyReader, Reader};
pub use record::{PyRecord, Record, RecordMutOp, RecordOp};
pub use writer::{PyWriter, Writer};

mod indexed_reader;
mod reader;
mod record;
mod writer;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyRecord>()?
        .add_class::<PyReader>()?
        .add_class::<PyIndexedReader>()?
        .add_class::<PyWriter>()?
        .finish();
    module.add("DEFAULT_LINE_WIDTH", DEFAULT_LINE_WIDTH)?;

    Ok(module)
}
