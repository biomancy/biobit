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
        module: &Bound<PyModule>,
    ) -> PyResult<()> {
        module.add_class::<PyLayout>()?;
        module.add_class::<PyLayout_Single>()?;
        module.add_class::<PyLayout_Paired>()?;

        let name = module.name()?;
        for typbj in [
            PyLayout_Single::type_object_bound(py),
            PyLayout_Paired::type_object_bound(py),
        ] {
            typbj.setattr("__module__", &name)?
        }
        Ok(())
    }

    fn __getnewargs__(&self, py: Python) -> PyObject {
        match self {
            PyLayout::Single { file } => (file,).into_py(py),
            PyLayout::Paired { orientation, files } => {
                let orientation = orientation.into_py(py);
                let files = (files.0.as_path().into_py(py), files.1.as_path().into_py(py));
                (orientation, files).into_py(py)
            }
        }
    }
}
