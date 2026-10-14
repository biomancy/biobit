use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyFrozenSet};

use super::input::{Input, input_to_python};
use crate::primitives::{PyMeta, define_id_binding};
use crate::provenance::sample::PySampleId;
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

type RsP5P7 = rs::provenance::library::p5p7::Library;
type RsP5P7Id = rs::provenance::library::p5p7::LibraryId;

define_id_binding!(PyP5P7Id, RsP5P7Id, "P5P7Id");

#[pyclass(from_py_object, frozen, eq, name = "P5P7")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyP5P7(pub Frozen<RsP5P7>);
frozen_wrapper!(PyP5P7, RsP5P7);
crate::utils::pickle::pickle_methods!(PyP5P7, RsP5P7, |value| value.0.as_ref());

#[pymethods]
impl PyP5P7 {
    #[new]
    #[pyo3(signature = (id, *, samples, input, meta=None, description=None))]
    fn new(
        id: PyP5P7Id,
        samples: Iterable<PySampleId>,
        input: Input,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let samples = samples.into_iter().map(|value| value.0.as_ref().clone());
        Ok(RsP5P7::new(
            id.0.as_ref().clone(),
            rs::NonEmpty::try_from_iter(samples)?,
            input.into(),
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )
        .into())
    }

    #[getter]
    fn id(&self) -> PyP5P7Id {
        self.0.map(RsP5P7::id).into()
    }

    #[getter]
    fn samples(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(RsP5P7::samples);
        let values = collection.iter_frozen().map(PySampleId::from);
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn input(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        input_to_python(py, self.0.as_ref().input())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(RsP5P7::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
