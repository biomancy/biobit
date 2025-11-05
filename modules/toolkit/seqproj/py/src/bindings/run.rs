use std::path::PathBuf;

use derive_more::From;
use pyo3::prelude::*;

use biobit_core_py::utils::ImportablePyModuleBuilder;

#[pyclass(eq, ord, hash, frozen, name = "Storage")]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From)]
pub enum PyStorage {
    SingleFastq { file: PathBuf },
    PairedFastq { file1: PathBuf, file2: PathBuf },
}

pub fn __biobit_initialize_complex_enum__(
    module: ImportablePyModuleBuilder,
) -> PyResult<ImportablePyModuleBuilder> {
    module
        .add_class::<PyStorage>()?
        .add_class::<PyStorage_SingleFastq>()?
        .add_class::<PyStorage_PairedFastq>()
}

#[pymethods]
impl PyStorage {
    fn __getnewargs__(&self, py: Python) -> PyResult<Py<PyAny>> {
        Ok(match self {
            PyStorage::SingleFastq { file } => (file,).into_pyobject(py)?.unbind().into(),
            PyStorage::PairedFastq { file1, file2 } => {
                (file1, file2).into_pyobject(py)?.unbind().into()
            }
        })
    }
}
