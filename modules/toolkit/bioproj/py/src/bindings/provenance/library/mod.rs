//! Library variants and their Python conversions.

mod input;
mod p5p7;

use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

use crate::primitives::define_family_conversions;

use rs::provenance::LibraryIdRef;

use input::{PyDna, PyRna};
pub use p5p7::{PyP5P7, PyP5P7Id};

define_family_conversions! {
    LibraryInput, LibraryIdInput {
        rust: rs::provenance::Library,
        id: rs::provenance::LibraryId,
        id_ref: LibraryIdRef,
        variants: {
            P5P7 => (PyP5P7, PyP5P7Id),
        }
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyP5P7Id>()?
        .add_class::<PyP5P7>()?
        .add_class::<PyDna>()?
        .add_class::<PyRna>()?
        .finish();
    module.add(
        "P5P7Input",
        module.getattr("DNA")?.bitor(module.getattr("RNA")?)?,
    )?;
    module.add("Library", module.getattr("P5P7")?)?;
    module.add("LibraryId", module.getattr("P5P7Id")?)?;
    module.add(
        "__all__",
        [
            "DNA",
            "P5P7",
            "RNA",
            "Library",
            "LibraryId",
            "P5P7Id",
            "P5P7Input",
        ],
    )?;
    Ok(module)
}
