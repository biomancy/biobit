use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PySequence;

use biobit_core_rs::loc::{AsSegment, Segment};

#[pyclass]
#[repr(transparent)]
#[derive(Debug, Into, From, Dissolve)]
pub struct IntoPySegment(pub Py<PySegment>);

impl<'py> FromPyObject<'py> for IntoPySegment {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let segment = if obj.is_instance_of::<PySegment>() {
            obj.downcast::<PySegment>()?.clone().unbind()
        } else {
            let seq = obj
                .downcast::<PySequence>()
                .map_err(|err| PyValueError::new_err(format!("Unknown segment: {}", err)))?;

            if seq.len()? != 2 {
                return Err(PyValueError::new_err(format!(
                    "Expected a sequence of length 2, got {}",
                    seq.len()?
                )));
            }

            let start = seq.get_item(0)?.extract::<i64>()?;
            let end = seq.get_item(1)?.extract::<i64>()?;

            Py::new(obj.py(), PySegment::new(start, end)?)?
        };
        Ok(IntoPySegment(segment))
    }
}

#[pyclass(name = "Segment")]
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, From, Into)]
pub struct PySegment {
    pub rs: Segment<i64>,
}

#[pymethods]
#[allow(clippy::len_without_is_empty)]
impl PySegment {
    #[new]
    pub fn new(start: i64, end: i64) -> PyResult<Self> {
        match Segment::new(start, end) {
            Ok(segment) => Ok(segment.into()),
            Err(_) => Err(PyValueError::new_err(format!(
                "[{}, {}) is not a valid segment",
                start, end
            ))),
        }
    }

    #[getter]
    pub fn start(&self) -> i64 {
        self.rs.start()
    }

    #[getter]
    pub fn end(&self) -> i64 {
        self.rs.end()
    }

    pub fn len(&self) -> i64 {
        self.rs.len()
    }

    pub fn contains(&self, pos: i64) -> bool {
        self.rs.contains(pos)
    }

    pub fn intersects(&self, py: Python, other: IntoPySegment) -> bool {
        self.rs.intersects(&other.0.borrow(py).rs)
    }

    pub fn touches(&self, py: Python, other: IntoPySegment) -> bool {
        self.rs.touches(&other.0.borrow(py).rs)
    }

    pub fn _extend(&mut self, left: i64, right: i64) -> PyResult<()> {
        if left < 0 || right < 0 {
            return Err(PyValueError::new_err(format!(
                "Left and right must be non-negative, got {} and {}",
                left, right
            )));
        }

        match self.rs.extend(left as u64, right as u64) {
            Some(_) => Ok(()),
            None => Err(PyValueError::new_err(format!(
                "Failed to extend segment {} by [{}, {}]",
                self.rs, left, right
            ))),
        }
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extend(mut slf: PyRefMut<PySegment>, left: i64, right: i64) -> PyResult<PyRefMut<Self>> {
        slf._extend(left, right)?;
        Ok(slf)
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extended(&self, left: i64, right: i64) -> PyResult<Self> {
        let mut cloned = *self;
        cloned._extend(left, right)?;

        Ok(cloned)
    }

    pub fn intersection(&self, py: Python, other: IntoPySegment) -> Option<Self> {
        self.rs.intersection(&other.0.borrow(py).rs).map(Self::from)
    }

    pub fn union(&self, py: Python, other: IntoPySegment) -> Option<Self> {
        self.rs.union(&other.0.borrow(py).rs).map(Self::from)
    }

    #[staticmethod]
    pub fn merge(py: Python, segments: Vec<IntoPySegment>) -> Vec<Self> {
        let segments = segments.into_iter().map(|s| s.0.borrow(py).rs).collect();
        Segment::merge(segments)
            .into_iter()
            .map(Self::from)
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("Segment[{}]", self.rs)
    }

    fn __str__(&self) -> String {
        self.rs.to_string()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __richcmp__(&self, py: Python, other: IntoPySegment, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.rs == other.0.borrow(py).rs,
            CompareOp::Ne => self.rs != other.0.borrow(py).rs,
            CompareOp::Lt => self.rs < other.0.borrow(py).rs,
            CompareOp::Le => self.rs <= other.0.borrow(py).rs,
            CompareOp::Gt => self.rs > other.0.borrow(py).rs,
            CompareOp::Ge => self.rs >= other.0.borrow(py).rs,
        }
    }
}

impl AsSegment for PySegment {
    type Idx = i64;

    fn start(&self) -> Self::Idx {
        self.start()
    }

    fn end(&self) -> Self::Idx {
        self.end()
    }
}
