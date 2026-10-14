use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use super::unit::PyUnitId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

define_id_binding!(
    PyGroupContrastId,
    rs::design::GroupContrastId,
    "GroupContrastId"
);

#[pyclass(from_py_object, frozen, eq, name = "GroupContrast")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyGroupContrast(pub Frozen<rs::design::GroupContrast>);

frozen_wrapper!(PyGroupContrast, rs::design::GroupContrast);
crate::utils::pickle::pickle_methods!(PyGroupContrast, rs::design::GroupContrast, |value| value
    .0
    .as_ref());

#[pymethods]
impl PyGroupContrast {
    #[new]
    #[pyo3(signature = (id, *, name, control, treatment, meta=None, description=None))]
    fn new(
        id: PyGroupContrastId,
        name: String,
        control: Iterable<PyUnitId>,
        treatment: Iterable<PyUnitId>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let control = control.into_iter().map(|value| value.0.as_ref().clone());
        let treatment = treatment.into_iter().map(|value| value.0.as_ref().clone());
        Ok(rs::design::GroupContrast::new(
            id.0.as_ref().clone(),
            rs::Text::new(name)?,
            rs::NonEmpty::try_from_iter(control)?,
            rs::NonEmpty::try_from_iter(treatment)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )?
        .into())
    }

    #[getter]
    fn id(&self) -> PyGroupContrastId {
        self.0.map(rs::design::GroupContrast::id).into()
    }

    #[getter]
    fn name(&self) -> &str {
        self.0.as_ref().name().as_str()
    }

    #[getter]
    fn control(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(rs::design::GroupContrast::control);
        let values = collection.iter_frozen().map(PyUnitId::from);
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn treatment(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(rs::design::GroupContrast::treatment);
        let values = collection.iter_frozen().map(PyUnitId::from);
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::design::GroupContrast::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
