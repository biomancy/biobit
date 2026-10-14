use biobit_bioproj_rs as rs;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use crate::primitives::{PyMeta, define_id_binding};
use crate::provenance::AcquisitionIdInput;
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

define_id_binding!(PyUnitId, rs::design::UnitId, "UnitId");

#[pyclass(from_py_object, frozen, eq, name = "Unit")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyUnit(pub Frozen<rs::design::Unit>);

frozen_wrapper!(PyUnit, rs::design::Unit);

#[pymethods]
impl PyUnit {
    #[new]
    #[pyo3(signature = (id, *, acquisitions, meta=None, description=None))]
    fn new(
        id: PyUnitId,
        acquisitions: Iterable<AcquisitionIdInput>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let acquisitions = acquisitions.into_iter().map(Into::into);
        Ok(Self::from(rs::design::Unit::new(
            id.0.as_ref().clone(),
            rs::NonEmpty::try_from_iter(acquisitions)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )))
    }

    #[getter]
    fn id(&self) -> PyUnitId {
        self.0.map(rs::design::Unit::id).into()
    }

    #[getter]
    fn acquisitions(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(rs::design::Unit::acquisitions);
        let values = collection
            .iter_frozen()
            .map(|value| {
                AcquisitionIdInput::from_owner(&value, rs::provenance::AcquisitionId::as_ref)
                    .into_py_any(py)
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::design::Unit::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
