use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use super::source::PySourceId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

define_id_binding!(PySampleId, rs::provenance::SampleId, "SampleId");

#[pyclass(from_py_object, frozen, eq, name = "Sample")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PySample(pub Frozen<rs::provenance::Sample>);
frozen_wrapper!(PySample, rs::provenance::Sample);

#[pymethods]
impl PySample {
    #[new]
    #[pyo3(signature = (id, *, sources, meta=None, description=None))]
    fn new(
        id: PySampleId,
        sources: Iterable<PySourceId>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let sources = sources.into_iter().map(|value| value.0.as_ref().clone());
        Ok(rs::provenance::Sample::new(
            id.0.as_ref().clone(),
            rs::NonEmpty::try_from_iter(sources)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )
        .into())
    }

    #[getter]
    fn id(&self) -> PySampleId {
        self.0.map(rs::provenance::Sample::id).into()
    }

    #[getter]
    fn sources(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(rs::provenance::Sample::sources);
        let values = collection.iter_frozen().map(PySourceId::from);
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::provenance::Sample::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
