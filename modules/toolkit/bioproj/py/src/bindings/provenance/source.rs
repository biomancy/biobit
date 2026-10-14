use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use crate::primitives::{PyMeta, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};

define_id_binding!(PySourceId, rs::provenance::SourceId, "SourceId");

#[pyclass(from_py_object, frozen, eq, name = "Source")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PySource(pub Frozen<rs::provenance::Source>);
frozen_wrapper!(PySource, rs::provenance::Source);
crate::utils::pickle::pickle_methods!(PySource, rs::provenance::Source, |value| value.0.as_ref());

#[pymethods]
impl PySource {
    #[new]
    #[pyo3(signature = (id, *, organism=None, meta=None, description=None))]
    fn new(
        id: PySourceId,
        organism: Option<String>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        Ok(rs::provenance::Source::new(
            id.0.as_ref().clone(),
            organism.map(rs::Text::new).transpose()?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )
        .into())
    }

    #[getter]
    fn id(&self) -> PySourceId {
        self.0.map(rs::provenance::Source::id).into()
    }

    #[getter]
    fn organism(&self) -> Option<&str> {
        self.0.as_ref().organism().map(|value| value.as_str())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::provenance::Source::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
