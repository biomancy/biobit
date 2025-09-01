use std::path::PathBuf;

use derive_more::From;
use pyo3::prelude::*;

pub use biobit_core_py::ngs::PyMatesOrientation;
use biobit_core_py::utils::ImportablePyModuleBuilder;

#[pyclass(eq, ord, hash, frozen, name = "Layout")]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From)]
pub enum PyLayout {
    /// Single-end sequencing library
    Single { file: PathBuf },
    /// Paired-end library
    Paired {
        orientation: Option<PyMatesOrientation>,
        files: (PathBuf, PathBuf),
    },
}

pub fn __biobit_initialize_complex_enum__(
    module: ImportablePyModuleBuilder,
) -> PyResult<ImportablePyModuleBuilder> {
    module
        .add_class::<PyLayout>()?
        .add_class::<PyLayout_Single>()?
        .add_class::<PyLayout_Paired>()
}

#[pymethods]
impl PyLayout {
    fn __getnewargs__(&self, py: Python) -> PyResult<Py<PyAny>> {
        Ok(match self {
            PyLayout::Single { file } => (file,).into_pyobject(py)?.unbind().into(),
            PyLayout::Paired { orientation, files } => {
                let orientation = orientation.into_pyobject(py)?;
                let files = (files.0.as_path(), files.1.as_path()).into_pyobject(py)?;
                (orientation, files).into_pyobject(py)?.unbind().into()
            }
        })
    }
}
