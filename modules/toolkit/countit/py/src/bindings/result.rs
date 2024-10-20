use derive_getters::Dissolve;
use pyo3::prelude::*;

use biobit_core_py::loc::{AsSegment, PySegment};
pub use biobit_countit_rs::{Counts, Summary};

#[pyclass(get_all, name = "AlignmentsSummary")]
#[derive(Clone, Debug, PartialEq, Dissolve)]
pub struct PyAlignmentsSummary {
    resolved: u64,
    discarded: u64,
}

#[pyclass(get_all, name = "Stats")]
#[derive(Clone, Debug, Dissolve)]
pub struct PyStats {
    contig: String,
    segment: Py<PySegment>,
    time_s: f64,
    alignments: Py<PyAlignmentsSummary>,
}

impl IntoPy<PyStats> for Summary<String, usize> {
    fn into_py(self, py: Python<'_>) -> PyStats {
        let (contig, segment, time_s, alignments) = self.dissolve();
        let segment: PySegment = PySegment::new(segment.start() as i64, segment.end() as i64)
            .expect("Failed to convert segment");
        let alignments: PyAlignmentsSummary = PyAlignmentsSummary {
            resolved: alignments.resolved,
            discarded: alignments.discarded,
        };

        PyStats {
            contig,
            segment: Py::new(py, segment).expect("Failed to convert segment"),
            time_s,
            alignments: Py::new(py, alignments).expect("Failed to convert AlignmentsSummary"),
        }
    }
}

#[pymethods]
impl PyStats {
    pub fn __eq__(&self, py: Python, other: &PyStats) -> bool {
        self.contig == other.contig
            && *self.segment.borrow(py) == *other.segment.borrow(py)
            && self.time_s == other.time_s
            && *self.alignments.borrow(py) == *other.alignments.borrow(py)
    }
}

#[pyclass(get_all, name = "Counts")]
#[derive(Clone, Debug)]
pub struct PyCounts {
    source: PyObject,
    data: Vec<PyObject>,
    counts: Vec<f64>,
    stats: Vec<Py<PyStats>>,
}

impl IntoPy<PyCounts> for Counts<String, usize, f64, PyObject, PyObject> {
    fn into_py(self, py: Python<'_>) -> PyCounts {
        let (source, data, counts, stats) = self.dissolve();
        let stats: Vec<Py<PyStats>> = stats
            .into_iter()
            .map(|x| Py::new(py, x.into_py(py)))
            .collect::<PyResult<_>>()
            .expect("Failed to convert CountIt stats from Rust to Python");

        PyCounts {
            source,
            data,
            counts,
            stats,
        }
    }
}

#[pymethods]
impl PyCounts {
    pub fn __eq__(&self, other: &PyCounts) -> bool {
        self.source.is(&other.source)
            && self.data.len() == other.data.len()
            && self.stats.len() == other.stats.len()
            && self.counts == other.counts
            && self
                .data
                .iter()
                .zip(other.data.iter())
                .all(|(a, b)| a.is(b))
            && self
                .stats
                .iter()
                .zip(other.stats.iter())
                .all(|(a, b)| a.is(b))
    }
}
