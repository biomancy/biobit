use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

mod reader;
mod record;

pub use record::{PyBed12, PyBed3, PyBed4, PyBed5, PyBed6, PyBed8, PyBed9};

pub use reader::{PyBed3Reader, PyReader};

pub use biobit_io_rs::bed::{Reader, ReaderMutOp};

use crate::bed::reader::{
    PyBed12Reader, PyBed4Reader, PyBed5Reader, PyBed6Reader, PyBed8Reader, PyBed9Reader,
};
pub use biobit_io_rs::bed::{
    Bed12, Bed12MutOp, Bed12Op, Bed3, Bed3MutOp, Bed3Op, Bed4, Bed4MutOp, Bed4Op, Bed5, Bed5MutOp,
    Bed5Op, Bed6, Bed6MutOp, Bed6Op, Bed8, Bed8MutOp, Bed8Op, Bed9, Bed9MutOp, Bed9Op,
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
        // Plain readers
        .add_class::<PyReader>()?
        .add_class::<PyBed3Reader>()?
        .add_class::<PyBed4Reader>()?
        .add_class::<PyBed5Reader>()?
        .add_class::<PyBed6Reader>()?
        .add_class::<PyBed8Reader>()?
        .add_class::<PyBed9Reader>()?
        .add_class::<PyBed12Reader>()?
        .finish();

    Ok(module)
}
