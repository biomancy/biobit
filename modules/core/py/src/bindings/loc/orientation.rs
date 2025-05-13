use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{Display, From, Into};
use pyo3::{
    basic::CompareOp,
    exceptions::PyValueError,
    prelude::*,
    types::{PyInt, PyString},
};

use biobit_core_rs::loc::Orientation;

use super::strand::PyStrand;

use bitcode::{Decode, Encode};

#[pyclass]
#[repr(transparent)]
#[derive(Debug, Into, From, Dissolve)]
pub struct IntoPyOrientation(pub PyOrientation);

impl<'py> FromPyObject<'py> for IntoPyOrientation {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let orientation = if obj.is_instance_of::<PyOrientation>() {
            *obj.downcast::<PyOrientation>()?.get()
        } else if obj.is_instance_of::<PyInt>() {
            match obj.extract::<i32>()? {
                1 => PyOrientation::Forward,
                -1 => PyOrientation::Reverse,
                0 => PyOrientation::Dual,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Unknown orientation: {}",
                        obj
                    )));
                }
            }
        } else if obj.is_instance_of::<PyString>() {
            let value = obj.extract::<String>()?;
            match value.as_str() {
                "+" => PyOrientation::Forward,
                "-" => PyOrientation::Reverse,
                "=" => PyOrientation::Dual,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Unknown orientation: {}",
                        value
                    )));
                }
            }
        } else {
            return Err(PyValueError::new_err(format!(
                "Unknown orientation: {}",
                obj
            )));
        };
        Ok(IntoPyOrientation(orientation))
    }
}

#[pyclass(frozen, name = "Orientation")]
#[repr(transparent)]
#[derive(
    Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Display,
)]
pub struct PyOrientation(pub Orientation);

#[pymethods]
impl PyOrientation {
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Forward: PyOrientation = PyOrientation(Orientation::Forward);

    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Reverse: PyOrientation = PyOrientation(Orientation::Reverse);

    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Dual: PyOrientation = PyOrientation(Orientation::Dual);

    #[new]
    fn __new__(obj: IntoPyOrientation) -> PyResult<Self> {
        Ok(obj.0)
    }

    pub fn flipped(&self) -> Self {
        PyOrientation::from(self.0.flipped())
    }

    pub fn symbol(&self) -> &'static str {
        match self.0 {
            Orientation::Forward => "+",
            Orientation::Reverse => "-",
            Orientation::Dual => "=",
        }
    }

    pub fn to_strand(&self) -> PyResult<PyStrand> {
        let strand = self
            .0
            .try_into()
            .map_err(|_| PyValueError::new_err("Dual orientation has no corresponding strand"))?;

        Ok(PyStrand(strand))
    }

    fn __repr__(&self) -> &'static str {
        match self.0 {
            Orientation::Forward => "Orientation[+]",
            Orientation::Reverse => "Orientation[-]",
            Orientation::Dual => "Orientation[=]",
        }
    }

    fn __str__(&self) -> &'static str {
        self.symbol()
    }

    fn __richcmp__(&self, other: IntoPyOrientation, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => *self == other.0,
            CompareOp::Ne => *self != other.0,
            CompareOp::Lt => *self < other.0,
            CompareOp::Le => *self <= other.0,
            CompareOp::Gt => *self > other.0,
            CompareOp::Ge => *self >= other.0,
        }
    }
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __getnewargs__(&self) -> (&str,) {
        (self.symbol(),)
    }
}
