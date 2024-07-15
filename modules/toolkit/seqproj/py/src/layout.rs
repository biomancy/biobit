use std::path::PathBuf;

use derive_more::From;
use pyo3::prelude::*;

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
