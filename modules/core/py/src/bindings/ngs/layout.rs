use derive_more::From;
use pyo3::prelude::*;
use pyo3::PyTypeInfo;

use biobit_core_rs::ngs::Layout;

use super::mates_orientation::PyMatesOrientation;
use super::strandedness::PyStrandedness;

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
