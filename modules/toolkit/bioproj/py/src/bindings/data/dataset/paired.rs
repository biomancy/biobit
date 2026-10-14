use biobit_bioproj_rs as rs;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use crate::data::asset::PyAssetFastqId;
use crate::primitives::{PyMeta, define_id_binding};
use crate::provenance::PyIlluminaPairedEndSequencingId;
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

type RsPairedFastq = rs::data::dataset::fastq::Paired;
type RsPairedFastqId = rs::data::dataset::fastq::PairedId;

define_id_binding!(PyPairedFastqId, RsPairedFastqId, "PairedFastqId");

#[pyclass(from_py_object, frozen, eq, name = "PairedFastq")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyPairedFastq(pub Frozen<RsPairedFastq>);

frozen_wrapper!(PyPairedFastq, RsPairedFastq);
crate::utils::pickle::pickle_methods!(PyPairedFastq, RsPairedFastq, |value| value.0.as_ref());

#[pymethods]
impl PyPairedFastq {
    #[new]
    #[pyo3(signature = (id, *, acquisition, pairs, meta=None, description=None))]
    fn new(
        id: PyPairedFastqId,
        acquisition: PyRef<'_, PyIlluminaPairedEndSequencingId>,
        pairs: Iterable<(PyAssetFastqId, PyAssetFastqId)>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        let pairs = pairs
            .into_iter()
            .map(|(read1, read2)| {
                rs::data::dataset::fastq::Pair::new(
                    read1.0.as_ref().clone(),
                    read2.0.as_ref().clone(),
                )
            })
            .collect::<rs::Result<Vec<_>>>()?;
        Ok(RsPairedFastq::new(
            id.0.as_ref().clone(),
            acquisition.0.as_ref().clone(),
            rs::NonEmpty::try_from_iter(pairs)?,
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )?
        .into())
    }

    #[getter]
    fn id(&self) -> PyPairedFastqId {
        self.0.map(RsPairedFastq::id).into()
    }

    #[getter]
    fn acquisition(&self) -> PyIlluminaPairedEndSequencingId {
        self.0
            .map(|value| match value.acquisition() {
                rs::data::dataset::fastq::PairedInput::IlluminaPairedEndSequencing(id) => id,
            })
            .into()
    }

    #[getter]
    fn pairs(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
        let collection = self.0.map(RsPairedFastq::pairs);
        let values = collection.iter_frozen().map(|pair| {
            (
                PyAssetFastqId::from(pair.map(rs::data::dataset::fastq::Pair::read1)),
                PyAssetFastqId::from(pair.map(rs::data::dataset::fastq::Pair::read2)),
            )
        });
        Ok(PyFrozenSet::new(py, values)?.unbind())
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(RsPairedFastq::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
