use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use super::unit::PyUnitId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

define_id_binding!(PyUnitSetId, rs::design::UnitSetId, "UnitSetId");

#[pyclass(from_py_object, frozen, eq, name = "UnitSet")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyUnitSet(pub Frozen<rs::design::UnitSet>);

frozen_wrapper!(PyUnitSet, rs::design::UnitSet);
crate::utils::pickle::pickle_methods!(PyUnitSet, rs::design::UnitSet, |value| value.0.as_ref());

#[pymethods]
impl PyUnitSet {
    #[new]
    #[pyo3(signature = (id, *, name, units, meta=None, description=None))]
    fn new(
        id: PyUnitSetId,
        name: String,
        units: Iterable<PyUnitId>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let units = units.into_iter().map(|value| value.0.as_ref().clone());
        Ok(Self::from(rs::design::UnitSet::new(
            id.0.as_ref().clone(),
            rs::Text::new(name)?,
            rs::NonEmpty::try_from_iter(units)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )))
    }

    #[getter]
    fn id(&self) -> PyUnitSetId {
        self.0.map(rs::design::UnitSet::id).into()
    }

    #[getter]
    fn name(&self) -> &str {
        self.0.as_ref().name().as_str()
    }

    #[getter]
    fn units(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(rs::design::UnitSet::units);
        let values = collection.iter_frozen().map(PyUnitId::from);
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::design::UnitSet::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
