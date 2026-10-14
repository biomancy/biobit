//! Dataset variants and their Python conversions.

mod paired;
mod single;

use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

use crate::primitives::define_family_conversions;

use rs::data::DatasetIdRef;

pub use paired::{PyPairedFastq, PyPairedFastqId};
pub use single::{PyDatasetFastq, PyDatasetFastqId};

define_family_conversions! {
    DatasetInput, DatasetIdInput {
        rust: rs::data::Dataset,
        id: rs::data::DatasetId,
        id_ref: DatasetIdRef,
        variants: {
            Fastq => (PyDatasetFastq, PyDatasetFastqId),
            PairedFastq => (PyPairedFastq, PyPairedFastqId),
        }
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyDatasetFastqId>()?
        .add_class::<PyDatasetFastq>()?
        .add_class::<PyPairedFastqId>()?
        .add_class::<PyPairedFastq>()?
        .finish();
    module.add(
        "Dataset",
        module
            .getattr("Fastq")?
            .bitor(module.getattr("PairedFastq")?)?,
    )?;
    module.add(
        "DatasetId",
        module
            .getattr("FastqId")?
            .bitor(module.getattr("PairedFastqId")?)?,
    )?;
    module.add(
        "__all__",
        [
            "Dataset",
            "DatasetId",
            "Fastq",
            "FastqId",
            "PairedFastq",
            "PairedFastqId",
        ],
    )?;
    Ok(module)
}
