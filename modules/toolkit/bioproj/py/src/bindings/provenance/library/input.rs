use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use crate::primitives::{IntoPyStrandedness, PyStrandedness};

#[pyclass(from_py_object, frozen, eq, hash, name = "DNA")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PyDna;

#[pymethods]
impl PyDna {
    #[new]
    fn new() -> Self {
        Self
    }
}

#[pyclass(from_py_object, frozen, eq, hash, name = "RNA")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PyRna {
    strandedness: Option<rs::Strandedness>,
}

#[pymethods]
impl PyRna {
    #[new]
    #[pyo3(signature = (*, strandedness=None))]
    fn new(strandedness: Option<IntoPyStrandedness>) -> Self {
        Self {
            strandedness: strandedness.map(Into::into),
        }
    }

    #[getter]
    fn strandedness(&self) -> Option<PyStrandedness> {
        self.strandedness.map(Into::into)
    }
}

#[derive(FromPyObject)]
pub(super) enum Input {
    #[pyo3(transparent)]
    Dna(PyDna),
    #[pyo3(transparent)]
    Rna(PyRna),
}

impl From<Input> for rs::provenance::library::p5p7::Input {
    fn from(value: Input) -> Self {
        match value {
            Input::Dna(PyDna) => Self::Dna,
            Input::Rna(value) => Self::Rna {
                strandedness: value.strandedness,
            },
        }
    }
}

pub(super) fn input_to_python(
    py: Python<'_>,
    value: &rs::provenance::library::p5p7::Input,
) -> PyResult<Py<PyAny>> {
    match value {
        rs::provenance::library::p5p7::Input::Dna => Ok(Py::new(py, PyDna)?.into_any()),
        rs::provenance::library::p5p7::Input::Rna { strandedness } => Ok(Py::new(
            py,
            PyRna {
                strandedness: *strandedness,
            },
        )?
        .into_any()),
    }
}
