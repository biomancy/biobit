use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;

use rs::provenance::AcquisitionIdRef;

use super::library::PyP5P7Id;
use crate::primitives::{PyMeta, define_family_conversions, define_id_binding};
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

type RsSingleAcquisition = rs::provenance::acquisition::illumina::SingleEndSequencing;
type RsSingleAcquisitionId = rs::provenance::acquisition::illumina::SingleEndSequencingId;
type RsPairedAcquisition = rs::provenance::acquisition::illumina::PairedEndSequencing;
type RsPairedAcquisitionId = rs::provenance::acquisition::illumina::PairedEndSequencingId;

define_id_binding!(
    PyIlluminaSingleEndSequencingId,
    RsSingleAcquisitionId,
    "IlluminaSingleEndSequencingId"
);
define_id_binding!(
    PyIlluminaPairedEndSequencingId,
    RsPairedAcquisitionId,
    "IlluminaPairedEndSequencingId"
);

macro_rules! acquisition_binding {
    ($python:ident, $id:ident, $rust:ty, $name:literal) => {
        #[pyclass(from_py_object, frozen, eq, name = $name)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $python(pub Frozen<$rust>);
        frozen_wrapper!($python, $rust);
        crate::utils::pickle::pickle_methods!($python, $rust, |value| value.0.as_ref());

        #[pymethods]
        impl $python {
            #[new]
            #[pyo3(signature = (id, *, libraries, meta=None, description=None))]
            fn new(
                id: $id,
                libraries: Iterable<PyP5P7Id>,
                meta: Option<PyMeta>,
                description: Option<String>,
            ) -> PyBioprojResult<Self> {
                let libraries = libraries.into_iter().map(|value| value.0.as_ref().clone());
                Ok(<$rust>::new(
                    id.0.as_ref().clone(),
                    rs::NonEmpty::try_from_iter(libraries)?,
                    PyMeta::option_into_meta(meta),
                    description.map(rs::Text::new).transpose()?,
                )
                .into())
            }

            #[getter]
            fn id(&self) -> $id {
                self.0.map(<$rust>::id).into()
            }

            #[getter]
            fn libraries(&self, py: Python<'_>) -> PyResult<Py<PyFrozenSet>> {
                let collection = self.0.map(<$rust>::libraries);
                let values = collection.iter_frozen().map(PyP5P7Id::from);
                Ok(PyFrozenSet::new(py, values)?.unbind())
            }

            #[getter]
            fn meta(&self) -> PyMeta {
                self.0.map(<$rust>::meta).into()
            }

            #[getter]
            fn description(&self) -> Option<&str> {
                self.0.as_ref().description().map(|value| value.as_str())
            }
        }
    };
}

acquisition_binding!(
    PyIlluminaSingleEndSequencing,
    PyIlluminaSingleEndSequencingId,
    RsSingleAcquisition,
    "IlluminaSingleEndSequencing"
);
acquisition_binding!(
    PyIlluminaPairedEndSequencing,
    PyIlluminaPairedEndSequencingId,
    RsPairedAcquisition,
    "IlluminaPairedEndSequencing"
);

define_family_conversions! {
    AcquisitionInput, AcquisitionIdInput {
        rust: rs::provenance::Acquisition,
        id: rs::provenance::AcquisitionId,
        id_ref: AcquisitionIdRef,
        variants: {
            IlluminaSingleEndSequencing => (PyIlluminaSingleEndSequencing, PyIlluminaSingleEndSequencingId),
            IlluminaPairedEndSequencing => (PyIlluminaPairedEndSequencing, PyIlluminaPairedEndSequencingId),
        }
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyIlluminaSingleEndSequencingId>()?
        .add_class::<PyIlluminaSingleEndSequencing>()?
        .add_class::<PyIlluminaPairedEndSequencingId>()?
        .add_class::<PyIlluminaPairedEndSequencing>()?
        .finish();
    module.add(
        "Acquisition",
        module
            .getattr("IlluminaSingleEndSequencing")?
            .bitor(module.getattr("IlluminaPairedEndSequencing")?)?,
    )?;
    module.add(
        "AcquisitionId",
        module
            .getattr("IlluminaSingleEndSequencingId")?
            .bitor(module.getattr("IlluminaPairedEndSequencingId")?)?,
    )?;
    module.add(
        "__all__",
        [
            "Acquisition",
            "AcquisitionId",
            "IlluminaPairedEndSequencing",
            "IlluminaPairedEndSequencingId",
            "IlluminaSingleEndSequencing",
            "IlluminaSingleEndSequencingId",
        ],
    )?;
    Ok(module)
}
