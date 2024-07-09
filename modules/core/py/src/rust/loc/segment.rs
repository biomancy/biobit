use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use biobit_core_rs::loc::{Segment as RsSegment, SegmentLike};

#[pyclass(eq, ord)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, From, Into)]
pub struct Segment(RsSegment<i64>);

#[pymethods]
impl Segment {
    #[new]
    pub fn new(start: i64, end: i64) -> PyResult<Self> {
        match RsSegment::new(start, end) {
            Ok(segment) => Ok(Self(segment)),
            Err(_) => Err(PyValueError::new_err(format!(
                "[{}, {}) is not a valid segment",
                start, end
            ))),
        }
    }

    #[getter]
    pub fn start(&self) -> i64 {
        self.0.start()
    }

    #[getter]
    pub fn end(&self) -> i64 {
        self.0.end()
    }

    pub fn len(&self) -> i64 {
        self.0.len()
    }

    pub fn contains(&self, pos: i64) -> bool {
        self.0.contains(pos)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.0.intersects(&other.0)
    }

    pub fn touches(&self, other: &Self) -> bool {
        self.0.touches(&other.0)
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extend(&mut self, left: i64, right: i64) -> PyResult<()> {
        if left < 0 || right < 0 {
            return Err(PyValueError::new_err(format!(
                "Left and right must be non-negative, got {} and {}",
                left, right
            )));
        }

        match self.0.extend(left as u64, right as u64) {
            Some(_) => Ok(()),
            None => Err(PyValueError::new_err(format!(
                "Failed to extend segment {} by [{}, {}]",
                self.0, left, right
            ))),
        }
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extended(&self, left: i64, right: i64) -> PyResult<Self> {
        let mut cloned = self.clone();
        cloned.extend(left, right)?;
        Ok(cloned)
    }

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        self.0.intersection(&other.0).map(Self)
    }

    pub fn union(&self, other: &Self) -> Option<Self> {
        self.0.union(&other.0).map(Self)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl SegmentLike for Segment {
    type Idx = i64;

    fn start(&self) -> Self::Idx {
        self.start()
    }

    fn end(&self) -> Self::Idx {
        self.end()
    }
}
