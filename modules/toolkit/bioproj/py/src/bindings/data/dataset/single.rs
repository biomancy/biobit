use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use crate::data::asset::PyAssetFastqId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::provenance::PyIlluminaSingleEndSequencingId;
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

type RsDatasetFastq = rs::data::dataset::fastq::Single;
type RsDatasetFastqId = rs::data::dataset::fastq::SingleId;

define_id_binding!(PyDatasetFastqId, RsDatasetFastqId, "FastqId");

#[pyclass(from_py_object, frozen, eq, name = "Fastq")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyDatasetFastq(pub Frozen<RsDatasetFastq>);

frozen_wrapper!(PyDatasetFastq, RsDatasetFastq);
crate::utils::pickle::pickle_methods!(PyDatasetFastq, RsDatasetFastq, |value| value.0.as_ref());

#[pymethods]
impl PyDatasetFastq {
    #[new]
    #[pyo3(signature = (id, *, acquisition, assets, meta=None, description=None))]
    fn new(
        id: PyDatasetFastqId,
        acquisition: PyRef<'_, PyIlluminaSingleEndSequencingId>,
        assets: Iterable<PyAssetFastqId>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let assets = assets.into_iter().map(|value| value.0.as_ref().clone());
        Ok(Self::from(RsDatasetFastq::new(
            id.0.as_ref().clone(),
            acquisition.0.as_ref().clone(),
            rs::NonEmpty::try_from_iter(assets)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )))
    }

    #[getter]
    fn id(&self) -> PyDatasetFastqId {
        self.0.map(RsDatasetFastq::id).into()
    }

    #[getter]
    fn acquisition(&self) -> PyIlluminaSingleEndSequencingId {
        self.0
            .map(|value| match value.acquisition() {
                rs::data::dataset::fastq::SingleInput::IlluminaSingleEndSequencing(id) => id,
            })
            .into()
    }

    #[getter]
    fn assets(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(RsDatasetFastq::assets);
        let values = collection.iter_frozen().map(PyAssetFastqId::from);
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(RsDatasetFastq::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
