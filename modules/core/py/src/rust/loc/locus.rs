use std::fmt;
use std::fmt::Display;
use std::ops::Deref;

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::prelude::*;

use super::orientation::Orientation;
use super::segment::Segment;

// #[pyclass(eq, ord, get_all, set_all)]
// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, From, Into)]
#[pyclass(get_all, set_all)]
#[derive(Clone, Debug, Dissolve, From, Into)]
pub struct Locus {
    pub contig: String,
    pub segment: Py<Segment>,
    pub orientation: Orientation,
}

#[pymethods]
impl Locus {
    #[new]
    pub fn new(
        py: Python<'_>,
        contig: String,
        segment: Segment,
        orientation: Orientation,
    ) -> PyResult<Self> {
        let segment = Py::new(py, segment)?;
        Ok(Self {
            contig,
            segment,
            orientation,
        })
    }

    pub fn len(&self, py: Python<'_>) -> i64 {
        self.segment.borrow(py).len()
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extend(&mut self, left: i64, right: i64, py: Python<'_>) -> PyResult<()> {
        self.segment.borrow_mut(py).extend(left, right)
    }

    #[pyo3(signature = (left = 0, right = 0))]
    pub fn extended(&self, left: i64, right: i64, py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            segment: Py::new(py, self.segment.borrow(py).extended(left, right)?)?,
            ..self.clone()
        })
    }

    pub fn intersection(&self, other: &Self, py: Python<'_>) -> PyResult<Option<Self>> {
        if self.contig != other.contig {
            return Ok(None);
        }

        let other = other.segment.borrow(py);
        match self.segment.borrow(py).intersection(other.deref()) {
            None => Ok(None),
            Some(inter) => Ok(Some(Self {
                segment: Py::new(py, inter)?,
                ..self.clone()
            })),
        }
    }

    pub fn union(&self, other: &Self, py: Python<'_>) -> Option<Self> {
        if self.contig != other.contig {
            return None;
        }

        let other = other.segment.borrow(py);
        match self.segment.borrow(py).union(other.deref()) {
            None => None,
            Some(union) => Some(Self {
                segment: Py::new(py, union).unwrap(),
                ..self.clone()
            }),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    // fn __hash__(&self) -> u64 {
    //     let mut hasher = DefaultHasher::new();
    //     self.hash(&mut hasher);
    //     hasher.finish()
    // }
}

// impl LocusLike for Locus {
//     type Contig = String;
//     type Idx = i64;
//     type Segment = Segment;
//
//     fn contig(&self) -> &Self::Contig { &self.contig }
//
//     fn segment(&self) -> &Self::Segment {
//         Python::with_gil(|py| self.segment.borrow(py).deref())
//     }
//
//     fn orientation(&self) -> biobit_core_rs::loc::Orientation { self.orientation.into() }
//
//     fn as_locus(&self) -> RsLocus<Self::Contig, Self::Idx> {
//         RsLocus {
//             contig: self.contig.clone(),
//             segment: self.segment.as_segment(),
//             orientation: self.orientation().into(),
//         }
//     }
// }

impl Display for Locus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Python::with_gil(|py| {
            let segment = self.segment.borrow(py);
            write!(
                f,
                "{}:{}-{}[{}]",
                self.contig,
                segment.start(),
                segment.end(),
                self.orientation
            )
        })
    }
}
