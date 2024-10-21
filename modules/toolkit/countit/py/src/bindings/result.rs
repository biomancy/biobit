use biobit_core_py::loc::{AsSegment, PySegment};
pub use biobit_countit_rs::{Counts, PartitionMetrics, ResolutionOutcomes};
use derive_more::{From, Into};
use pyo3::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};

#[pyclass(frozen, eq, hash, get_all, name = "ResolutionOutcome")]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, From, Into)]
pub struct PyResolutionOutcome {
    resolved: f64,
    discarded: f64,
}

impl Hash for PyResolutionOutcome {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.resolved.to_bits().hash(state);
        self.discarded.to_bits().hash(state);
    }
}

impl IntoPy<PyResolutionOutcome> for ResolutionOutcomes<f64> {
    fn into_py(self, _py: Python<'_>) -> PyResolutionOutcome {
        PyResolutionOutcome {
            resolved: self.resolved,
            discarded: self.discarded,
        }
    }
}

#[pyclass(frozen, get_all, name = "Stats")]
#[derive(Clone, Debug, From, Into)]
pub struct PyPartitionMetrics {
    contig: String,
    segment: Py<PySegment>,
    time_s: f64,
    outcomes: Py<PyResolutionOutcome>,
}

impl IntoPy<PyPartitionMetrics> for PartitionMetrics<String, usize, f64> {
    fn into_py(self, py: Python<'_>) -> PyPartitionMetrics {
        let (contig, segment, time_s, alignments) = self.into();
        let segment: PySegment = PySegment::new(segment.start() as i64, segment.end() as i64)
            .expect("Failed to convert segment");
        let alignments: PyResolutionOutcome = PyResolutionOutcome {
            resolved: alignments.resolved,
            discarded: alignments.discarded,
        };

        PyPartitionMetrics {
            contig,
            segment: Py::new(py, segment).expect("Failed to convert segment"),
            time_s,
            outcomes: Py::new(py, alignments).expect("Failed to convert AlignmentsSummary"),
        }
    }
}

#[pymethods]
impl PyPartitionMetrics {
    pub fn __hash__(&self, py: Python) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.contig.hash(&mut hasher);
        self.segment.borrow(py).rs.hash(&mut hasher);
        self.time_s.to_bits().hash(&mut hasher);
        self.outcomes.borrow(py).hash(&mut hasher);
        hasher.finish()
    }

    pub fn __eq__(&self, py: Python, other: &PyPartitionMetrics) -> bool {
        self.contig == other.contig
            && *self.segment.borrow(py) == *other.segment.borrow(py)
            && self.time_s == other.time_s
            && *self.outcomes.borrow(py) == *other.outcomes.borrow(py)
    }
}

#[pyclass(get_all, name = "Counts")]
#[derive(Clone, Debug, From, Into)]
pub struct PyCounts {
    source: PyObject,
    data: Vec<PyObject>,
    counts: Vec<f64>,
    stats: Vec<Py<PyPartitionMetrics>>,
}

impl IntoPy<PyCounts> for Counts<String, usize, f64, PyObject, PyObject> {
    fn into_py(self, py: Python<'_>) -> PyCounts {
        let (source, data, counts, stats) = self.into();
        let stats: Vec<Py<PyPartitionMetrics>> = stats
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

// #[pymethods]
// impl PyCounts {
//     pub fn __hash__(&self, py: Python) -> u64 {
//         let mut hasher = DefaultHasher::new();
//
//         self.source.hash(&mut hasher);
//         self.data.iter().for_each(|x| x.hash(&mut hasher));
//         self.stats.iter().for_each(|x| x.borrow(py).hash(&mut hasher));
//         self.counts.to_bits().hash(&mut hasher);
//         hasher.finish()
//     }
//
//
//     pub fn __eq__(&self, other: &PyCounts, py: Python) -> bool {
//
//
//
//
//         self.source.is(&other.source)
//             && self.data.len() == other.data.len()
//             && self.stats.len() == other.stats.len()
//             && self.counts == other.counts
//             && self
//                 .data
//                 .iter()
//                 .zip(other.data.iter())
//                 .all(|(a, b)| a.bind(py), b))
//             && self
//                 .stats
//                 .iter()
//                 .zip(other.stats.iter())
//                 .all(|(a, b)| a.is(b))
//     }
// }
