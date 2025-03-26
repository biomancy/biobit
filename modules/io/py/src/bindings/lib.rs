use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub use biobit_io_rs::{ReadRecord, WriteRecord};

pub mod bam;
pub mod bed;
pub mod fasta;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&bam::construct(py, &format!("{name}.bam"))?)?
        .add_submodule(&fasta::construct(py, &format!("{name}.fasta"))?)?
        .add_submodule(&bed::construct(py, &format!("{name}.bed"))?)?
        .finish();

    Ok(module)
}
