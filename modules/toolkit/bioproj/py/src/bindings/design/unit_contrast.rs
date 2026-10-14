use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use super::unit::PyUnitId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};

define_id_binding!(
    PyUnitContrastId,
    rs::design::UnitContrastId,
    "UnitContrastId"
);

#[pyclass(from_py_object, frozen, eq, name = "UnitContrast")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyUnitContrast(pub Frozen<rs::design::UnitContrast>);

frozen_wrapper!(PyUnitContrast, rs::design::UnitContrast);
crate::utils::pickle::pickle_methods!(PyUnitContrast, rs::design::UnitContrast, |value| value
    .0
    .as_ref());

#[pymethods]
impl PyUnitContrast {
    #[new]
    #[pyo3(signature = (id, *, name, control, treatment, meta=None, description=None))]
    fn new(
        id: PyUnitContrastId,
        name: String,
        control: PyRef<'_, PyUnitId>,
        treatment: PyRef<'_, PyUnitId>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        Ok(rs::design::UnitContrast::new(
            id.0.as_ref().clone(),
            rs::Text::new(name)?,
            control.0.as_ref().clone(),
            treatment.0.as_ref().clone(),
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )?
        .into())
    }

    #[getter]
    fn id(&self) -> PyUnitContrastId {
        self.0.map(rs::design::UnitContrast::id).into()
    }

    #[getter]
    fn name(&self) -> &str {
        self.0.as_ref().name().as_str()
    }

    #[getter]
    fn control(&self) -> PyUnitId {
        self.0.map(rs::design::UnitContrast::control).into()
    }

    #[getter]
    fn treatment(&self) -> PyUnitId {
        self.0.map(rs::design::UnitContrast::treatment).into()
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::design::UnitContrast::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
