use std::fmt;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::orientation::{IntoPyOrientation, PyOrientation};
use super::segment::{IntoPySegment, PySegment};

#[derive(Debug, Into, From, Dissolve)]
pub struct IntoPyLocus(pub Py<PyLocus>);

impl FromPyObject<'_> for IntoPyLocus {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        let locus = if obj.is_instance_of::<PyLocus>() {
            obj.downcast::<PyLocus>()?.clone().unbind()
        } else if obj.is_instance_of::<PyTuple>() {
            let obj = obj.downcast::<PyTuple>()?;
            if obj.len() != 3 {
                return Err(PyValueError::new_err(format!(
                    "Expected a tuple of length 3, got {}",
                    obj.len()
                )));
            }

            let contig = obj.get_item(0)?.extract::<String>()?;
            let segment = obj.get_item(1)?.extract::<IntoPySegment>()?;
            let orientation = obj.get_item(2)?.extract::<IntoPyOrientation>()?;

            Py::new(obj.py(), PyLocus::new(contig, segment, orientation)?)?
        } else {
            return Err(PyValueError::new_err(format!("Unknown locus: {:?}", obj)));
        };

        Ok(locus.into())
    }
}

#[pyclass(get_all, name = "Locus")]
#[derive(Clone, Debug, Dissolve)]
pub struct PyLocus {
    pub contig: String,
    pub segment: Py<PySegment>,
    pub orientation: PyOrientation,
}

#[pymethods]
impl PyLocus {
    #[new]
    pub fn new(
        contig: String,
        segment: IntoPySegment,
        orientation: IntoPyOrientation,
    ) -> PyResult<Self> {
        Ok(Self {
            contig,
            segment: segment.0,
            orientation: orientation.0,
        })
    }

    #[setter]
    pub fn set_contig(&mut self, contig: String) {
        self.contig = contig;
    }

    #[setter]
    pub fn set_segment(&mut self, segment: IntoPySegment) {
        self.segment = segment.0;
    }

    #[setter]
    pub fn set_orientation(&mut self, orientation: IntoPyOrientation) {
        self.orientation = orientation.0;
    }

    pub fn len(&self, py: Python<'_>) -> i64 {
        self.segment.borrow(py).len()
    }

    pub fn flip(mut slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf.orientation = slf.orientation.flipped();
        slf
    }

    pub fn flipped(&self, py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            contig: self.contig.clone(),
            segment: Py::new(py, *self.segment.borrow(py))?,
            orientation: self.orientation.flipped(),
        })
    }

    // pub fn contains(&self, py: Python<'_>, pos: i64) -> bool {
    //     self.segment.borrow(py).contains(pos)
    // }
    //
    // pub fn intersects(&self, other: PyLocusLike) -> bool {
    //     let borrow = other.0.borrow();
    //     let py = borrow.py();
    //
    //     if self.contig != borrow.contig
    //         || *self.orientation.borrow(py) != *borrow.orientation.borrow(py)
    //     {
    //         return false;
    //     }
    //
    //     let result = self
    //         .segment
    //         .borrow(py)
    //         .rs
    //         .intersects(&borrow.segment.borrow(py).rs);
    //     result
    // }
    //
    // pub fn touches(&self, other: PyLocusLike) -> bool {
    //     let borrow = other.0.borrow();
    //     let py = borrow.py();
    //
    //     if self.contig != borrow.contig
    //         || *self.orientation.borrow(py) != *borrow.orientation.borrow(py)
    //     {
    //         return false;
    //     }
    //
    //     let result = self
    //         .segment
    //         .borrow(py)
    //         .rs
    //         .touches(&borrow.segment.borrow(py).rs);
    //     result
    // }
    //
    // #[pyo3(signature = (left = 0, right = 0))]
    // pub fn extend<'py>(
    //     slf: PyRefMut<'py, Self>,
    //     py: Python<'py>,
    //     left: i64,
    //     right: i64,
    // ) -> PyResult<PyRefMut<'py, Self>> {
    //     slf.segment.borrow_mut(py)._extend(left, right)?;
    //     Ok(slf)
    // }
    //
    // #[pyo3(signature = (left = 0, right = 0))]
    // pub fn extended(&self, left: i64, right: i64, py: Python<'_>) -> PyResult<Self> {
    //     Ok(Self {
    //         segment: Py::new(py, self.segment.borrow(py).extended(left, right)?)?,
    //         ..self.clone()
    //     })
    // }
    //
    // pub fn intersection(&self, other: PyLocusLike) -> PyResult<Option<Self>> {
    //     let borrow = other.0.borrow();
    //     let py = borrow.py();
    //
    //     if self.contig != borrow.contig {
    //         return Ok(None);
    //     }
    //
    //     let intersection = self
    //         .segment
    //         .borrow(py)
    //         .rs
    //         .intersection(&borrow.segment.borrow(py).rs);
    //
    //     match intersection {
    //         None => Ok(None),
    //         Some(rs) => Ok(Some(Self {
    //             segment: Py::new(py, PySegment { rs })?,
    //             ..self.clone()
    //         })),
    //     }
    // }
    //
    // pub fn union(&self, other: PyLocusLike) -> PyResult<Option<Self>> {
    //     let borrow = other.0.borrow();
    //     let py = borrow.py();
    //
    //     if self.contig != borrow.contig {
    //         return Ok(None);
    //     }
    //
    //     let union = self
    //         .segment
    //         .borrow(py)
    //         .rs
    //         .union(&borrow.segment.borrow(py).rs);
    //
    //     match union {
    //         None => Ok(None),
    //         Some(rs) => Ok(Some(Self {
    //             segment: Py::new(py, PySegment { rs })?,
    //             ..self.clone()
    //         })),
    //     }
    // }

    fn __repr__(&self) -> String {
        format!("Locus[{}]", self)
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __hash__(&self, py: Python) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.contig.hash(&mut hasher);
        self.segment.borrow(py).hash(&mut hasher);
        self.orientation.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __richcmp__(&self, py: Python, other: IntoPyLocus, op: CompareOp) -> bool {
        let borrow = other.0.borrow(py);

        let this = (&self.contig, self.orientation, self.segment.borrow(py).rs);
        let that = (
            &borrow.contig,
            borrow.orientation,
            borrow.segment.borrow(py).rs,
        );

        match op {
            CompareOp::Eq => this == that,
            CompareOp::Ne => this != that,
            CompareOp::Lt => this < that,
            CompareOp::Le => this <= that,
            CompareOp::Gt => this > that,
            CompareOp::Ge => this >= that,
        }
    }
}

impl Display for PyLocus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Python::with_gil(|py| {
            let segment = self.segment.borrow(py);
            write!(
                f,
                "{}:{}-{}({})",
                self.contig,
                segment.start(),
                segment.end(),
                self.orientation
            )
        })
    }
}
