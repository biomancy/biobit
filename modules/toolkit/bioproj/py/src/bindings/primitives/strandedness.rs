use biobit_bioproj_rs::Strandedness;
use derive_more::{From, Into};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

#[pyclass(from_py_object, frozen, eq, ord, name = "Strandedness")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, Into)]
pub struct PyStrandedness(pub Strandedness);

/// Opts an API input into accepting a strandedness value or its full name.
/// Python output remains a strict `Strandedness`, including Layout getters.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, Into)]
pub struct IntoPyStrandedness(pub Strandedness);

impl<'a, 'py> FromPyObject<'a, 'py> for IntoPyStrandedness {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = obj.cast::<PyStrandedness>() {
            return Ok(Self(value.get().0));
        }
        if let Ok(value) = obj.cast::<PyString>() {
            return match value.to_str()? {
                "Forward" => Ok(Self(Strandedness::Forward)),
                "Reverse" => Ok(Self(Strandedness::Reverse)),
                "Unstranded" => Ok(Self(Strandedness::Unstranded)),
                value => Err(PyValueError::new_err(format!(
                    "Invalid strandedness {value:?}; expected Forward, Reverse, or Unstranded"
                ))),
            };
        }
        Err(PyTypeError::new_err(
            "expected Strandedness or its full name",
        ))
    }
}

impl From<PyStrandedness> for IntoPyStrandedness {
    fn from(value: PyStrandedness) -> Self {
        Self(value.0)
    }
}

#[pymethods]
impl PyStrandedness {
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Forward: Self = Self(Strandedness::Forward);

    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Reverse: Self = Self(Strandedness::Reverse);

    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Unstranded: Self = Self(Strandedness::Unstranded);

    #[new]
    fn new(strandedness: IntoPyStrandedness) -> Self {
        Self(strandedness.0)
    }

    fn __repr__(&self) -> &'static str {
        match self.0 {
            Strandedness::Forward => "Strandedness[Forward]",
            Strandedness::Reverse => "Strandedness[Reverse]",
            Strandedness::Unstranded => "Strandedness[Unstranded]",
        }
    }

    fn __str__(&self) -> &'static str {
        self.0.as_str()
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        PyString::new(py, self.0.as_str()).hash()
    }

    fn __getnewargs__(&self) -> (&'static str,) {
        (self.0.as_str(),)
    }
}
