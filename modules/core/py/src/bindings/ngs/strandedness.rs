use std::hash::{DefaultHasher, Hash, Hasher};

use derive_more::{Display, From, Into};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyString;

use biobit_core_rs::ngs::Strandedness;

#[derive(Debug, Into, From)]
pub struct IntoPyStrandness(PyStrandedness);

impl<'py> FromPyObject<'py> for IntoPyStrandness {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let strandedness = if obj.is_instance_of::<PyStrandedness>() {
            *obj.downcast::<PyStrandedness>()?.get()
        } else if obj.is_instance_of::<PyString>() {
            match obj.extract::<String>()?.as_str() {
                "F" => PyStrandedness::Forward,
                "R" => PyStrandedness::Reverse,
                "U" => PyStrandedness::Unstranded,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Unknown strandedness: {}",
                        obj
                    )));
                }
            }
        } else {
            return Err(PyValueError::new_err(format!(
                "Unknown strandedness: {}",
                obj
            )));
        };
        Ok(strandedness.into())
    }
}
#[pyclass(frozen, name = "Strandedness")]
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Display)]
pub struct PyStrandedness(pub Strandedness);

#[pymethods]
impl PyStrandedness {
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Forward: PyStrandedness = PyStrandedness(Strandedness::Forward);
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Reverse: PyStrandedness = PyStrandedness(Strandedness::Reverse);
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Unstranded: PyStrandedness = PyStrandedness(Strandedness::Unstranded);

    #[new]
    pub fn __new__(strandedness: IntoPyStrandness) -> PyResult<Self> {
        Ok(strandedness.0)
    }

    pub fn symbol(&self) -> &'static str {
        self.0.symbol()
    }

    fn __repr__(&self) -> &'static str {
        match self.0 {
            Strandedness::Forward => "Strandedness[F]",
            Strandedness::Reverse => "Strandedness[R]",
            Strandedness::Unstranded => "Strandedness[U]",
        }
    }

    fn __str__(&self) -> &'static str {
        self.symbol()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: IntoPyStrandness, op: CompareOp) -> bool {
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
