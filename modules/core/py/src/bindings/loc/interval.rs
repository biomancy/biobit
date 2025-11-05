use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PySequence;

use biobit_core_rs::loc::{Interval, IntervalOp};

use biobit_core_rs::num::PrimInt;
use bitcode::{Decode, Encode};
use pyo3::BoundObject;

#[pyclass]
#[repr(transparent)]
#[derive(Debug, Into, From, Dissolve)]
pub struct IntoPyInterval(pub Py<PyInterval>);

impl<'a, 'py> FromPyObject<'a, 'py> for IntoPyInterval {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let interval = if obj.is_instance_of::<PyInterval>() {
            obj.cast::<PyInterval>()?.into_bound().unbind()
        } else {
            let seq = obj
                .cast::<PySequence>()
                .map_err(|err| PyValueError::new_err(format!("Unknown interval: {}", err)))?;

            if seq.len()? != 2 {
                return Err(PyValueError::new_err(format!(
                    "Expected a sequence of length 2, got {}",
                    seq.len()?
                )));
            }

            let start = seq.get_item(0)?.extract::<i64>()?;
            let end = seq.get_item(1)?.extract::<i64>()?;

            Py::new(obj.py(), PyInterval::new(start, end)?)?
        };
        Ok(IntoPyInterval(interval))
    }
}

impl IntoPyInterval {
    pub fn extract_rs<T: PrimInt>(self, py: Python) -> Option<Interval<T>> {
        self.0.borrow(py).rs.cast()
    }

    pub fn extract_py(self, py: Python) -> PyInterval {
        self.0.borrow(py).rs.into()
    }
}

#[pyclass(name = "Interval")]
#[repr(transparent)]
#[derive(
    Decode, Encode, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, From, Into,
)]
pub struct PyInterval {
    pub rs: Interval<i64>,
}

#[pymethods]
#[allow(clippy::len_without_is_empty)]
impl PyInterval {
    #[new]
    pub fn new(start: i64, end: i64) -> PyResult<Self> {
        match Interval::new(start, end) {
            Ok(interval) => Ok(interval.into()),
            Err(_) => Err(PyValueError::new_err(format!(
                "[{}, {}) is not a valid interval",
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

    pub fn intersects(&self, py: Python, other: IntoPyInterval) -> bool {
        self.rs.intersects(&other.0.borrow(py).rs)
    }

    pub fn touches(&self, py: Python, other: IntoPyInterval) -> bool {
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
                "Failed to extend interval {} by [{}, {}]",
                self.rs, left, right
            ))),
        }
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extend(
        mut slf: PyRefMut<PyInterval>,
        left: i64,
        right: i64,
    ) -> PyResult<PyRefMut<Self>> {
        slf._extend(left, right)?;
        Ok(slf)
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extended(&self, left: i64, right: i64) -> PyResult<Self> {
        let mut cloned = *self;
        cloned._extend(left, right)?;

        Ok(cloned)
    }

    pub fn intersection(&self, py: Python, other: IntoPyInterval) -> Option<Self> {
        self.rs.intersection(&other.0.borrow(py).rs).map(Self::from)
    }

    pub fn union(&self, py: Python, other: IntoPyInterval) -> Option<Self> {
        self.rs.union(&other.0.borrow(py).rs).map(Self::from)
    }

    #[staticmethod]
    pub fn merge(py: Python, intervals: Vec<IntoPyInterval>) -> Vec<Self> {
        let mut intervals: Vec<_> = intervals.into_iter().map(|s| s.0.borrow(py).rs).collect();
        Interval::merge(&mut intervals)
            .into_iter()
            .map(Self::from)
            .collect()
    }

    #[staticmethod]
    pub fn merge_within(py: Python, intervals: Vec<IntoPyInterval>, distance: u64) -> Vec<Self> {
        let mut intervals: Vec<_> = intervals.into_iter().map(|s| s.0.borrow(py).rs).collect();
        Interval::merge_within(&mut intervals, distance as i64)
            .into_iter()
            .map(Self::from)
            .collect()
    }

    #[staticmethod]
    pub fn subtract(
        py: Python,
        source: Vec<IntoPyInterval>,
        exclude: Vec<IntoPyInterval>,
    ) -> Vec<Self> {
        let mut source: Vec<_> = source.into_iter().map(|s| s.0.borrow(py).rs).collect();
        let mut exclude: Vec<_> = exclude.into_iter().map(|s| s.0.borrow(py).rs).collect();
        Interval::subtract(&mut source, &mut exclude)
            .into_iter()
            .map(Self::from)
            .collect()
    }

    #[staticmethod]
    pub fn overlap(py: Python, left: Vec<IntoPyInterval>, right: Vec<IntoPyInterval>) -> Vec<Self> {
        let mut source: Vec<_> = left.into_iter().map(|s| s.0.borrow(py).rs).collect();
        let mut exclude: Vec<_> = right.into_iter().map(|s| s.0.borrow(py).rs).collect();
        Interval::overlap(&mut source, &mut exclude)
            .into_iter()
            .map(Self::from)
            .collect()
    }

    #[staticmethod]
    pub fn overlaps(
        py: Python,
        source: Vec<IntoPyInterval>,
        overlap: Vec<IntoPyInterval>,
    ) -> Vec<bool> {
        let source: Vec<_> = source.into_iter().map(|s| s.0.borrow(py).rs).collect();
        let mut overlap: Vec<_> = overlap.into_iter().map(|s| s.0.borrow(py).rs).collect();
        Interval::overlaps(&source, &mut overlap)
    }

    fn __repr__(&self) -> String {
        format!("Interval[{}]", self.rs)
    }

    fn __str__(&self) -> String {
        self.rs.to_string()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __richcmp__(&self, py: Python, other: IntoPyInterval, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.rs == other.0.borrow(py).rs,
            CompareOp::Ne => self.rs != other.0.borrow(py).rs,
            CompareOp::Lt => self.rs < other.0.borrow(py).rs,
            CompareOp::Le => self.rs <= other.0.borrow(py).rs,
            CompareOp::Gt => self.rs > other.0.borrow(py).rs,
            CompareOp::Ge => self.rs >= other.0.borrow(py).rs,
        }
    }

    pub fn __getnewargs__(&self) -> (i64, i64) {
        (self.start(), self.end())
    }
}

impl IntervalOp for PyInterval {
    type Idx = i64;

    fn start(&self) -> Self::Idx {
        self.start()
    }

    fn end(&self) -> Self::Idx {
        self.end()
    }
}
