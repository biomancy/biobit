use std::hash::{DefaultHasher, Hash, Hasher};

use derive_more::{Display, From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyString;

use biobit_core_rs::ngs::MatesOrientation;

#[derive(Debug, Into, From)]
pub struct IntoPyMatesOrientation(PyMatesOrientation);

impl<'a, 'py> FromPyObject<'a, 'py> for IntoPyMatesOrientation {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let mates_orientation = if obj.is_instance_of::<PyMatesOrientation>() {
            *obj.cast::<PyMatesOrientation>()?.get()
        } else if obj.is_instance_of::<PyString>() {
            match obj.extract::<String>()?.as_str() {
                "I" => PyMatesOrientation::Inward,
                x => {
                    return Err(PyValueError::new_err(format!(
                        "Unknown mates orientation: {x}",
                    )));
                }
            }
        } else {
            return Err(PyValueError::new_err(format!(
                "Unknown mates orientation: {}",
                obj.str()?
            )));
        };
        Ok(mates_orientation.into())
    }
}

#[pyclass(frozen, name = "MatesOrientation")]
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Display)]
pub struct PyMatesOrientation(pub MatesOrientation);

#[pymethods]
impl PyMatesOrientation {
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Inward: PyMatesOrientation = PyMatesOrientation(MatesOrientation::Inward);

    #[new]
    pub fn __new__(mates_orientation: IntoPyMatesOrientation) -> PyResult<Self> {
        Ok(mates_orientation.0)
    }

    pub fn symbol(&self) -> &'static str {
        self.0.symbol()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> &'static str {
        match self.0 {
            MatesOrientation::Inward => "MatesOrientation[I]",
        }
    }

    fn __str__(&self) -> &'static str {
        match self.0 {
            MatesOrientation::Inward => "I",
        }
    }

    fn __richcmp__(&self, other: IntoPyMatesOrientation, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => *self == other.0,
            CompareOp::Ne => *self != other.0,
            CompareOp::Lt => *self < other.0,
            CompareOp::Le => *self <= other.0,
            CompareOp::Gt => *self > other.0,
            CompareOp::Ge => *self >= other.0,
        }
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.symbol(),)
    }
}
