//! Data records, indexes, and Python module registration.

mod asset;
mod dataset;
mod index;

use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::PyMapping;

use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

use asset::AssetInput;
use dataset::DatasetInput;
use index::{PyAssetIndex, PyDatasetIndex};

#[pyclass(from_py_object, frozen, eq, name = "Data")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyData(pub Frozen<rs::data::Data>);
frozen_wrapper!(PyData, rs::data::Data);
crate::utils::pickle::pickle_methods!(PyData, rs::data::Data, |value| value.0.as_ref());

#[pymethods]
impl PyData {
    #[new]
    #[pyo3(signature = (*, assets, datasets))]
    fn new(
        assets: Iterable<AssetInput>,
        datasets: Iterable<DatasetInput>,
    ) -> PyBioprojResult<Self> {
        Ok(rs::data::Data::new(
            assets.into_iter().map(Into::into),
            datasets.into_iter().map(Into::into),
        )?
        .into())
    }

    #[getter]
    fn assets(&self) -> PyAssetIndex {
        PyAssetIndex(self.0.clone())
    }

    #[getter]
    fn datasets(&self) -> PyDatasetIndex {
        PyDatasetIndex(self.0.clone())
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    PyMapping::register::<PyAssetIndex>(py)?;
    PyMapping::register::<PyDatasetIndex>(py)?;
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyData>()?
        .add_class::<PyAssetIndex>()?
        .add_class::<PyDatasetIndex>()?
        .add_submodule(&asset::construct(py, &format!("{name}.asset"))?)?
        .add_submodule(&dataset::construct(py, &format!("{name}.dataset"))?)?
        .finish();
    module.add(
        "__all__",
        ["AssetIndex", "Data", "DatasetIndex", "asset", "dataset"],
    )?;
    Ok(module)
}
