use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{Display, From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyInt, PyString};

use biobit_core_rs::loc::{Orientation, Strand};

use super::orientation::PyOrientation;

use bitcode::{Decode, Encode};

#[derive(Debug, Dissolve, Into, From)]
pub struct IntoPyStrand(PyStrand);

impl<'a, 'py> FromPyObject<'a, 'py> for IntoPyStrand {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let pystrand = if obj.is_instance_of::<PyStrand>() {
            *obj.cast::<PyStrand>()?.get()
        } else if obj.is_instance_of::<PyInt>() {
            match obj.extract::<i32>()? {
                1 => PyStrand::Forward,
                -1 => PyStrand::Reverse,
                x => return Err(PyValueError::new_err(format!("Unknown strand: {x}"))),
            }
        } else if obj.is_instance_of::<PyString>() {
            let value = obj.extract::<String>()?;
            match value.as_str() {
                "+" => PyStrand::Forward,
                "-" => PyStrand::Reverse,
                x => return Err(PyValueError::new_err(format!("Unknown strand: {x}"))),
            }
        } else {
            return Err(PyValueError::new_err(format!(
                "Unknown strand: {}",
                obj.str()?
            )));
        };
        Ok(pystrand.into())
    }
}

#[pyclass(frozen, name = "Strand")]
#[repr(transparent)]
#[derive(
    Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Display,
)]
pub struct PyStrand(pub Strand);

#[pymethods]
impl PyStrand {
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Forward: PyStrand = PyStrand(Strand::Forward);
    #[classattr]
    #[allow(non_upper_case_globals)]
    pub const Reverse: PyStrand = PyStrand(Strand::Reverse);

    #[new]
    pub fn __new__(strand: IntoPyStrand) -> PyResult<Self> {
        Ok(strand.0)
    }

    pub fn flipped(&self) -> Self {
        PyStrand(self.0.flipped())
    }

    pub fn symbol(&self) -> &'static str {
        match self.0 {
            Strand::Forward => "+",
            Strand::Reverse => "-",
        }
    }

    pub fn to_orientation(&self) -> PyOrientation {
        <Strand as Into<Orientation>>::into(self.0).into()
    }
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> &'static str {
        match self.0 {
            Strand::Forward => "Strand[+]",
            Strand::Reverse => "Strand[-]",
        }
    }

    fn __str__(&self) -> &'static str {
        self.symbol()
    }

    fn __richcmp__(&self, other: IntoPyStrand, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => *self == other.0,
            CompareOp::Ne => *self != other.0,
            CompareOp::Lt => *self < other.0,
            CompareOp::Le => *self <= other.0,
            CompareOp::Gt => *self > other.0,
            CompareOp::Ge => *self >= other.0,
        }
    }

    pub fn __getnewargs__(&self) -> (&str,) {
        (self.symbol(),)
    }
}
