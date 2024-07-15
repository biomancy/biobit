use derive_getters::Dissolve;
use pyo3::prelude::*;

use biobit_core_py::loc::{AsSegment, PySegment};
pub use biobit_countit_rs::{Counts, Stats};

#[pyclass(get_all, name = "Stats")]
#[derive(Clone, Debug, Dissolve)]
pub struct PyStats {
    contig: String,
    segment: Py<PySegment>,
    time_s: f64,
    inside_annotation: f64,
    outside_annotation: f64,
}

impl IntoPy<PyStats> for Stats<String, usize, f64> {
    fn into_py(self, py: Python<'_>) -> PyStats {
        let (contig, segment, time_s, inside_annotation, outside_annotation) = self.dissolve();
        let segment: PySegment = PySegment::new(segment.start() as i64, segment.end() as i64)
            .expect("Failed to convert segment");

        PyStats {
            contig,
            segment: Py::new(py, segment).expect("Failed to convert segment"),
            time_s,
            inside_annotation,
            outside_annotation,
        }
    }
}

#[pymethods]
impl PyStats {
    pub fn __eq__(&self, py: Python, other: &PyStats) -> bool {
        self.contig == other.contig
            && *self.segment.borrow(py) == *other.segment.borrow(py)
            && self.time_s == other.time_s
            && self.inside_annotation == other.inside_annotation
            && self.outside_annotation == other.outside_annotation
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
