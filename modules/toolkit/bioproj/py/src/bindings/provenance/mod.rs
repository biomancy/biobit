//! Provenance records, indexes, and Python module registration.

mod acquisition;
mod index;
mod library;
mod sample;
mod source;

use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::PyMapping;

use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

use acquisition::AcquisitionInput;
pub use acquisition::{
    AcquisitionIdInput, PyIlluminaPairedEndSequencingId, PyIlluminaSingleEndSequencingId,
};
use index::{PyAcquisitionIndex, PyLibraryIndex, PySampleIndex, PySourceIndex};
use library::LibraryInput;
use sample::PySample;
use source::PySource;

#[pyclass(from_py_object, frozen, eq, name = "Provenance")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyProvenance(pub Frozen<rs::provenance::Provenance>);
frozen_wrapper!(PyProvenance, rs::provenance::Provenance);

#[pymethods]
impl PyProvenance {
    #[new]
    #[pyo3(signature = (*, sources, samples, libraries, acquisitions))]
    fn new(
        sources: Iterable<PySource>,
        samples: Iterable<PySample>,
        libraries: Iterable<LibraryInput>,
        acquisitions: Iterable<AcquisitionInput>,
    ) -> PyBioprojResult<Self> {
        Ok(rs::provenance::Provenance::new(
            sources.into_iter().map(|value| value.0.as_ref().clone()),
            samples.into_iter().map(|value| value.0.as_ref().clone()),
            libraries.into_iter().map(Into::into),
            acquisitions.into_iter().map(Into::into),
        )?
        .into())
    }

    #[getter]
    fn sources(&self) -> PySourceIndex {
        PySourceIndex(self.0.clone())
    }

    #[getter]
    fn samples(&self) -> PySampleIndex {
        PySampleIndex(self.0.clone())
    }

    #[getter]
    fn libraries(&self) -> PyLibraryIndex {
        PyLibraryIndex(self.0.clone())
    }

    #[getter]
    fn acquisitions(&self) -> PyAcquisitionIndex {
        PyAcquisitionIndex(self.0.clone())
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    PyMapping::register::<PySourceIndex>(py)?;
    PyMapping::register::<PySampleIndex>(py)?;
    PyMapping::register::<PyLibraryIndex>(py)?;
    PyMapping::register::<PyAcquisitionIndex>(py)?;

    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<source::PySourceId>()?
        .add_class::<PySource>()?
        .add_class::<sample::PySampleId>()?
        .add_class::<PySample>()?
        .add_class::<PyProvenance>()?
        .add_class::<PySourceIndex>()?
        .add_class::<PySampleIndex>()?
        .add_class::<PyLibraryIndex>()?
        .add_class::<PyAcquisitionIndex>()?
        .add_submodule(&library::construct(py, &format!("{name}.library"))?)?
        .add_submodule(&acquisition::construct(py, &format!("{name}.acquisition"))?)?
        .finish();
    module.add(
        "__all__",
        [
            "AcquisitionIndex",
            "LibraryIndex",
            "Provenance",
            "Sample",
            "SampleId",
            "SampleIndex",
            "Source",
            "SourceId",
            "SourceIndex",
            "acquisition",
            "library",
        ],
    )?;
    Ok(module)
}
