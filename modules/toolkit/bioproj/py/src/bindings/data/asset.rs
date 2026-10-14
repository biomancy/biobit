use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

use rs::data::AssetIdRef;

use crate::primitives::{PyFilePath, PyMeta, define_family_conversions, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};

type RsAssetFastq = rs::data::asset::fastq::Fastq;
type RsAssetFastqId = rs::data::asset::fastq::FastqId;

define_id_binding!(PyAssetFastqId, RsAssetFastqId, "FastqId");

#[pyclass(from_py_object, frozen, eq, name = "Fastq")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyAssetFastq(pub Frozen<RsAssetFastq>);

frozen_wrapper!(PyAssetFastq, RsAssetFastq);
crate::utils::pickle::pickle_methods!(PyAssetFastq, RsAssetFastq, |value| value.0.as_ref());

#[pymethods]
impl PyAssetFastq {
    #[new]
    #[pyo3(signature = (id, *, location, meta=None, description=None))]
    fn new(
        id: PyAssetFastqId,
        location: PyFilePath,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        Ok(Self::from(RsAssetFastq::new(
            id.0.as_ref().clone(),
            location.0.as_ref().clone(),
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )))
    }

    #[getter]
    fn id(&self) -> PyAssetFastqId {
        self.0.map(RsAssetFastq::id).into()
    }

    #[getter]
    fn location(&self) -> PyFilePath {
        self.0.map(RsAssetFastq::location).into()
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(RsAssetFastq::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}

define_family_conversions! {
    AssetInput, AssetIdInput {
        rust: rs::data::Asset,
        id: rs::data::AssetId,
        id_ref: AssetIdRef,
        variants: {
            Fastq => (PyAssetFastq, PyAssetFastqId),
        }
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyAssetFastqId>()?
        .add_class::<PyAssetFastq>()?
        .finish();
    module.add("Asset", module.getattr("Fastq")?)?;
    module.add("AssetId", module.getattr("FastqId")?)?;
    module.add("__all__", ["Asset", "AssetId", "Fastq", "FastqId"])?;
    Ok(module)
}
