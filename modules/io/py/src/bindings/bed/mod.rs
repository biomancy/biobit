use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

mod reader;
mod record;
mod writer;

pub use record::{
    Bed12, Bed12MutOp, Bed12Op, Bed3, Bed3MutOp, Bed3Op, Bed4, Bed4MutOp, Bed4Op, Bed5, Bed5MutOp,
    Bed5Op, Bed6, Bed6MutOp, Bed6Op, Bed8, Bed8MutOp, Bed8Op, Bed9, Bed9MutOp, Bed9Op, PyBed12,
    PyBed3, PyBed4, PyBed5, PyBed6, PyBed8, PyBed9,
};

pub use reader::{
    PyBed12Reader, PyBed3Reader, PyBed4Reader, PyBed5Reader, PyBed6Reader, PyBed8Reader,
    PyBed9Reader, PyReader, Reader,
};

pub use writer::{
    PyBed12Writer, PyBed3Writer, PyBed4Writer, PyBed5Writer, PyBed6Writer, PyBed8Writer,
    PyBed9Writer, PyWriter, Writer,
};

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        // BED records
        .add_class::<PyBed3>()?
        .add_class::<PyBed4>()?
        .add_class::<PyBed5>()?
        .add_class::<PyBed6>()?
        .add_class::<PyBed8>()?
        .add_class::<PyBed9>()?
        .add_class::<PyBed12>()?
        // Readers
        .add_class::<PyReader>()?
        .add_class::<PyBed3Reader>()?
        .add_class::<PyBed4Reader>()?
        .add_class::<PyBed5Reader>()?
        .add_class::<PyBed6Reader>()?
        .add_class::<PyBed8Reader>()?
        .add_class::<PyBed9Reader>()?
        .add_class::<PyBed12Reader>()?
        // Writers
        .add_class::<PyWriter>()?
        .add_class::<PyBed3Writer>()?
        .add_class::<PyBed4Writer>()?
        .add_class::<PyBed5Writer>()?
        .add_class::<PyBed6Writer>()?
        .add_class::<PyBed8Writer>()?
        .add_class::<PyBed9Writer>()?
        .add_class::<PyBed12Writer>()?
        .finish();

    Ok(module)
}
