use biobit_bioproj_rs::Layout;
use derive_more::{From, Into};
use pyo3::prelude::*;

use super::{IntoPyMatesOrientation, IntoPyStrandedness, PyMatesOrientation, PyStrandedness};

/// Native layout storage shared by the concrete Python variant classes.
#[pyclass(from_py_object, frozen, subclass, eq, ord, hash, name = "Layout")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, Into)]
pub struct PyLayout(pub Layout);

#[pymethods]
impl PyLayout {
    #[getter]
    fn strandedness(&self) -> PyStrandedness {
        match self.0 {
            Layout::Single { strandedness } | Layout::Paired { strandedness, .. } => {
                strandedness.into()
            }
        }
    }

    fn __getnewargs__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(match self.0 {
            Layout::Single { strandedness } => (PyStrandedness(strandedness),)
                .into_pyobject(py)?
                .unbind()
                .into(),
            Layout::Paired {
                strandedness,
                orientation,
            } => (
                PyStrandedness(strandedness),
                PyMatesOrientation(orientation),
            )
                .into_pyobject(py)?
                .unbind()
                .into(),
        })
    }
}

// Variant classes carry no additional Rust data. Their constructors create the
// matching native enum in the base, preserving isinstance and pattern matching.
#[pyclass(skip_from_py_object, frozen, extends = PyLayout, name = "Single")]
pub(super) struct PyLayoutSingle;

#[pymethods]
impl PyLayoutSingle {
    #[new]
    fn new(strandedness: IntoPyStrandedness) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyLayout(Layout::Single {
            strandedness: strandedness.into(),
        }))
        .add_subclass(Self)
    }

    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str,) = ("strandedness",);
}

#[pyclass(skip_from_py_object, frozen, extends = PyLayout, name = "Paired")]
pub(super) struct PyLayoutPaired;

#[pymethods]
impl PyLayoutPaired {
    #[new]
    fn new(
        strandedness: IntoPyStrandedness,
        orientation: IntoPyMatesOrientation,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyLayout(Layout::Paired {
            strandedness: strandedness.into(),
            orientation: orientation.into(),
        }))
        .add_subclass(Self)
    }

    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("strandedness", "orientation");

    #[getter]
    fn orientation(slf: PyRef<'_, Self>) -> PyMatesOrientation {
        match slf.into_super().0 {
            Layout::Paired { orientation, .. } => orientation.into(),
            Layout::Single { .. } => unreachable!("Paired constructor creates a paired layout"),
        }
    }
}
