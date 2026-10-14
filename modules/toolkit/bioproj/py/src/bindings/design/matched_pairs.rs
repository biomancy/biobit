use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use super::unit::PyUnitId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

define_id_binding!(
    PyMatchedPairsId,
    rs::design::MatchedPairsId,
    "MatchedPairsId"
);

#[pyclass(from_py_object, frozen, eq, name = "MatchedPairs")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyMatchedPairs(pub Frozen<rs::design::MatchedPairs>);

frozen_wrapper!(PyMatchedPairs, rs::design::MatchedPairs);
crate::utils::pickle::pickle_methods!(PyMatchedPairs, rs::design::MatchedPairs, |value| value
    .0
    .as_ref());

#[pymethods]
impl PyMatchedPairs {
    #[new]
    #[pyo3(signature = (id, *, name, pairs, meta=None, description=None))]
    fn new(
        id: PyMatchedPairsId,
        name: String,
        pairs: Iterable<(PyUnitId, PyUnitId)>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let pairs = pairs
            .into_iter()
            .map(|(first, second)| {
                rs::design::MatchedPair::new(first.0.as_ref().clone(), second.0.as_ref().clone())
            })
            .collect::<rs::Result<Vec<_>>>()?;
        Ok(rs::design::MatchedPairs::new(
            id.0.as_ref().clone(),
            rs::Text::new(name)?,
            rs::NonEmpty::try_from_iter(pairs)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )?
        .into())
    }

    #[getter]
    fn id(&self) -> PyMatchedPairsId {
        self.0.map(rs::design::MatchedPairs::id).into()
    }

    #[getter]
    fn name(&self) -> &str {
        self.0.as_ref().name().as_str()
    }

    #[getter]
    fn pairs(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(rs::design::MatchedPairs::pairs);
        let values = collection.iter_frozen().map(|pair| {
            (
                PyUnitId::from(pair.map(rs::design::MatchedPair::first)),
                PyUnitId::from(pair.map(rs::design::MatchedPair::second)),
            )
        });
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::design::MatchedPairs::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
