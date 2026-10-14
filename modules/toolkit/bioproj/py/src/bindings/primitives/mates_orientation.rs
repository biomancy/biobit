use biobit_bioproj_rs::MatesOrientation;
use derive_more::{From, Into};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

#[pyclass(from_py_object, frozen, eq, ord, name = "MatesOrientation")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, Into)]
pub struct PyMatesOrientation(pub MatesOrientation);

crate::utils::pickle::pickle_methods!(PyMatesOrientation, MatesOrientation, |value| &value.0);

/// Opts an API input into accepting a mates orientation value or its full name.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, Into)]
pub struct IntoPyMatesOrientation(pub MatesOrientation);

impl<'a, 'py> FromPyObject<'a, 'py> for IntoPyMatesOrientation {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = obj.cast::<PyMatesOrientation>() {
            return Ok(Self(value.get().0));
        }
        if let Ok(value) = obj.cast::<PyString>() {
            return match value.to_str()? {
                "Inward" => Ok(Self(MatesOrientation::Inward)),
                value => Err(PyValueError::new_err(format!(
                    "Invalid mates orientation {value:?}; expected Inward"
                ))),
            };
        }
        Err(PyTypeError::new_err(
            "expected MatesOrientation or its full name",
        ))
    }
}

impl From<PyMatesOrientation> for IntoPyMatesOrientation {
    fn from(value: PyMatesOrientation) -> Self {
        Self(value.0)
    }
}

#[pymethods]
impl PyMatesOrientation {
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Inward: Self = Self(MatesOrientation::Inward);

    #[new]
    fn new(mates_orientation: IntoPyMatesOrientation) -> Self {
        Self(mates_orientation.0)
    }

    fn __repr__(&self) -> &'static str {
        match self.0 {
            MatesOrientation::Inward => "MatesOrientation[Inward]",
        }
    }

    fn __str__(&self) -> &'static str {
        self.0.as_str()
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        PyString::new(py, self.0.as_str()).hash()
    }
}
