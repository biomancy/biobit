use std::path::PathBuf;

use derive_more::From;
use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use biobit_core_py::ngs::PyMatesOrientation;

// pub use biobit_seqproj_rs::Layout;

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

#[pymethods]
impl PyLayout {
    #[staticmethod]
    pub fn __biobit_initialize_complex_enum__(
        py: Python,
        path: &str,
        module: &Bound<PyModule>,
    ) -> PyResult<()> {
        module.add_class::<PyLayout>()?;
        module.add_class::<PyLayout_Single>()?;
        module.add_class::<PyLayout_Paired>()?;

        for typbj in [
            PyLayout_Single::type_object(py),
            PyLayout_Paired::type_object(py),
        ] {
            typbj.setattr("__module__", path)?
        }
        Ok(())
    }

    fn __getnewargs__(&self, py: Python) -> PyResult<PyObject> {
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
