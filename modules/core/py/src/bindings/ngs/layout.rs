use derive_more::From;
use pyo3::prelude::*;

use super::mates_orientation::PyMatesOrientation;
use super::strandedness::PyStrandedness;
use crate::utils::ImportablePyModuleBuilder;
use biobit_core_rs::ngs::Layout;

#[pyclass(eq, ord, hash, frozen, name = "Layout")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From)]
pub enum PyLayout {
    /// Single-end sequencing library
    Single { strandedness: PyStrandedness },
    /// Paired-end library
    Paired {
        strandedness: PyStrandedness,
        orientation: PyMatesOrientation,
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
    fn __getnewargs__(&self, py: Python) -> PyResult<PyObject> {
        Ok(match self {
            PyLayout::Single { strandedness } => {
                (*strandedness,).into_pyobject(py)?.unbind().into()
            }
            PyLayout::Paired {
                strandedness,
                orientation,
            } => (*strandedness, *orientation)
                .into_pyobject(py)?
                .unbind()
                .into(),
        })
    }
}

impl From<PyLayout> for Layout {
    fn from(value: PyLayout) -> Self {
        match value {
            PyLayout::Single { strandedness } => Layout::Single {
                strandedness: strandedness.into(),
            },
            PyLayout::Paired {
                strandedness,
                orientation,
            } => Layout::Paired {
                strandedness: strandedness.into(),
                orientation: orientation.into(),
            },
        }
    }
}
