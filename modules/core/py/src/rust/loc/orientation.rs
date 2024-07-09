use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyString};

use biobit_core_rs::loc::Orientation as RsOrientation;

use super::strand::Strand;

#[pyclass(eq, ord)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum Orientation {
    Forward = 1,
    Reverse = -1,
    Dual = 0,
}

#[pymethods]
impl Orientation {
    #[new]
    fn __new__<'a, 'py>(obj: &Bound<'a, PyAny>) -> PyResult<Self> {
        if obj.is_instance_of::<PyInt>() {
            match obj.extract::<i32>()? {
                1 => Ok(Orientation::Forward),
                -1 => Ok(Orientation::Reverse),
                0 => Ok(Orientation::Dual),
                _ => Err(PyValueError::new_err(format!(
                    "Unknown orientation: {}",
                    obj
                ))),
            }
        } else if obj.is_instance_of::<PyString>() {
            let value = obj.extract::<String>()?;
            match value.as_str() {
                "+" => Ok(Orientation::Forward),
                "-" => Ok(Orientation::Reverse),
                "=" => Ok(Orientation::Dual),
                _ => Err(PyValueError::new_err(format!(
                    "Unknown orientation: {}",
                    value
                ))),
            }
        } else if obj.is_instance_of::<Orientation>() {
            obj.extract()
        } else {
            Err(PyValueError::new_err(format!(
                "Unknown orientation: {}",
                obj
            )))
        }
    }

    pub fn flip(&mut self) {
        *self = match self {
            Orientation::Forward => Orientation::Reverse,
            Orientation::Reverse => Orientation::Forward,
            Orientation::Dual => Orientation::Dual,
        };
    }

    pub fn flipped(&self) -> Self {
        match self {
            Orientation::Forward => Orientation::Reverse,
            Orientation::Reverse => Orientation::Forward,
            Orientation::Dual => Orientation::Dual,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Orientation::Forward => "+",
            Orientation::Reverse => "-",
            Orientation::Dual => "=",
        }
    }

    pub fn to_strand(&self) -> PyResult<Strand> {
        match self {
            Orientation::Forward => Ok(Strand::Forward),
            Orientation::Reverse => Ok(Strand::Reverse),
            Orientation::Dual => Err(PyValueError::new_err(
                "Dual orientation has no corresponding strand",
            )),
        }
    }

    fn __repr__(&self) -> &'static str {
        match self {
            Orientation::Forward => "Orientation[+]",
            Orientation::Reverse => "Orientation[-]",
            Orientation::Dual => "Orientation[=]",
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

impl From<Orientation> for RsOrientation {
    fn from(value: Orientation) -> Self {
        match value {
            Orientation::Forward => RsOrientation::Forward,
            Orientation::Reverse => RsOrientation::Reverse,
            Orientation::Dual => RsOrientation::Dual,
        }
    }
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}
