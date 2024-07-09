use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyString};

use biobit_core_rs::loc::Strand as RsStrand;

use super::orientation::Orientation;

#[pyclass(eq, ord)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum Strand {
    Forward = 1,
    Reverse = -1,
}

#[pymethods]
impl Strand {
    #[new]
    fn __new__<'a, 'py>(obj: &Bound<'a, PyAny>) -> PyResult<Self> {
        if obj.is_instance_of::<PyInt>() {
            match obj.extract::<i32>()? {
                1 => Ok(Strand::Forward),
                -1 => Ok(Strand::Reverse),
                _ => Err(PyValueError::new_err(format!("Unknown strand: {}", obj))),
            }
        } else if obj.is_instance_of::<PyString>() {
            let value = obj.extract::<String>()?;
            match value.as_str() {
                "+" => Ok(Strand::Forward),
                "-" => Ok(Strand::Reverse),
                _ => Err(PyValueError::new_err(format!("Unknown strand: {}", value))),
            }
        } else if obj.is_instance_of::<Strand>() {
            obj.extract()
        } else {
            Err(PyValueError::new_err(format!("Unknown strand: {}", obj)))
        }
    }

    pub fn flip(&mut self) {
        *self = match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        };
    }

    pub fn flipped(&self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Forward => "+",
            Self::Reverse => "-",
        }
    }

    pub fn to_orientation(&self) -> Orientation {
        match self {
            Self::Forward => Orientation::Forward,
            Self::Reverse => Orientation::Reverse,
        }
    }

    fn __repr__(&self) -> &'static str {
        match self {
            Self::Forward => "Strand[+]",
            Self::Reverse => "Strand[-]",
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
}

impl From<Strand> for RsStrand {
    fn from(value: Strand) -> Self {
        match value {
            Strand::Forward => RsStrand::Forward,
            Strand::Reverse => RsStrand::Reverse,
        }
    }
}
