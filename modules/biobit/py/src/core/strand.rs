use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::PyTypeInfo;
use pyo3::types::{PyInt, PyString};


#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strand {
    Forward,
    Reverse,
}

#[pymethods]
impl Strand {
    #[new]
    fn __new__<'a, 'py>(obj: Bound<'a, PyAny>) -> PyResult<Self> {
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
    fn __str__(&self) -> &'static str {
        match self {
            Strand::Forward => "+",
            Strand::Reverse => "-"
        }
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            CompareOp::Eq => (self == other).into_py(py),
            CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    // _strand_impl!(Strand);
}


// from enum import Enum
// from typing import Literal, TypeVar, Generic
// StrandLike = Strand | Literal["+", "-", 1, -1]
//
// _T = TypeVar("_T")
//
//
// @define(slots=True, frozen=True, eq=True, repr=True, hash=True)
// class Stranded(Generic[_T]):
//     fwd: _T = field()
//     rev: _T = field()
//
//     def with_rev(self, value: _T) -> 'Stranded[_T]':
//         return Stranded(self.fwd, value)
//
//     def with_fwd(self, value: _T) -> 'Stranded[_T]':
//         return Stranded(value, self.rev)
//
//     def __iter__(self):
//         yield from (self.fwd, self.rev)