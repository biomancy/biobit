use biobit_core_py::loc::{IntervalOp, PyInterval};
use biobit_core_py::utils::type_hint_class_getitem;
pub use biobit_countit_rs::{Counts, PartitionMetrics, ResolutionOutcomes};
use derive_more::{From, Into};
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::hash::{DefaultHasher, Hash, Hasher};

#[pyclass(frozen, eq, hash, name = "ResolutionOutcome")]
#[derive(Clone, Debug, PartialEq, PartialOrd, From, Into)]
#[repr(transparent)]
pub struct PyResolutionOutcome {
    rs: ResolutionOutcomes<f64>,
}

impl Hash for PyResolutionOutcome {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rs.resolved.to_bits().hash(state);
        self.rs.discarded.to_bits().hash(state);
    }
}

#[pymethods]
impl PyResolutionOutcome {
    #[getter]
    pub fn resolved(&self) -> f64 {
        self.rs.resolved
    }

    #[getter]
    pub fn discarded(&self) -> f64 {
        self.rs.discarded
    }
}

#[pyclass(frozen, get_all, name = "PartitionMetrics")]
#[derive(Clone, Debug, From, Into)]
pub struct PyPartitionMetrics {
    contig: String,
    interval: Py<PyInterval>,
    time_s: f64,
    outcomes: Py<PyResolutionOutcome>,
}

impl From<PartitionMetrics<String, usize, f64>> for PyPartitionMetrics {
    fn from(metrics: PartitionMetrics<String, usize, f64>) -> Self {
        let (contig, interval, time_s, alignments) = metrics.into();
        let interval: PyInterval = PyInterval::new(interval.start() as i64, interval.end() as i64)
            .expect("Failed to convert interval");
        let alignments: PyResolutionOutcome = alignments.into();

        Python::attach(|py| PyPartitionMetrics {
            contig,
            interval: Py::new(py, interval).expect("Failed to convert interval"),
            time_s,
            outcomes: Py::new(py, alignments).expect("Failed to convert AlignmentsSummary"),
        })
    }
}

#[pymethods]
impl PyPartitionMetrics {
    pub fn __hash__(&self, py: Python) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.contig.hash(&mut hasher);
        self.interval.borrow(py).rs.hash(&mut hasher);
        self.time_s.to_bits().hash(&mut hasher);
        self.outcomes.borrow(py).hash(&mut hasher);
        hasher.finish()
    }

    pub fn __eq__(&self, py: Python, other: &PyPartitionMetrics) -> bool {
        self.contig == other.contig
            && *self.interval.borrow(py) == *other.interval.borrow(py)
            && self.time_s == other.time_s
            && *self.outcomes.borrow(py) == *other.outcomes.borrow(py)
    }
}

#[pyclass(get_all, name = "Counts")]
#[derive(Clone, Debug, From, Into)]
pub struct PyCounts {
    source: Py<PyAny>,
    elements: Vec<Py<PyAny>>,
    counts: Vec<f64>,
    partitions: Vec<Py<PyPartitionMetrics>>,
}

impl From<Counts<'_, String, usize, f64, Py<PyAny>, Py<PyAny>>> for PyCounts {
    fn from(counts: Counts<String, usize, f64, Py<PyAny>, Py<PyAny>>) -> Self {
        let (source, elements, counts, stats) = counts.into();
        Python::attach(|py| {
            let partitions: Vec<Py<PyPartitionMetrics>> = stats
                .into_iter()
                .map(|x| Py::new(py, PyPartitionMetrics::from(x)))
                .collect::<PyResult<_>>()
                .expect("Failed to convert CountIt stats from Rust to Python");
            PyCounts {
                source,
                elements: elements.to_vec(),
                counts,
                partitions,
            }
        })
    }
}

#[pymethods]
impl PyCounts {
    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: Py<PyAny>) -> PyResult<Py<PyAny>> {
        type_hint_class_getitem(cls, args)
    }

    // pub fn __hash__(&self, py: Python) -> u64 {
    //     let mut hasher = DefaultHasher::new();
    //
    //     self.source.hash(&mut hasher);
    //     self.data.iter().for_each(|x| x.hash(&mut hasher));
    //     self.stats.iter().for_each(|x| x.borrow(py).hash(&mut hasher));
    //     self.counts.to_bits().hash(&mut hasher);
    //     hasher.finish()
    // }
    //
    //
    // pub fn __eq__(&self, other: &PyCounts, py: Python) -> bool {
    //
    //
    //
    //
    //     self.source.is(&other.source)
    //         && self.data.len() == other.data.len()
    //         && self.stats.len() == other.stats.len()
    //         && self.counts == other.counts
    //         && self
    //             .data
    //             .iter()
    //             .zip(other.data.iter())
    //             .all(|(a, b)| a.bind(py), b))
    //         && self
    //             .stats
    //             .iter()
    //             .zip(other.stats.iter())
    //             .all(|(a, b)| a.is(b))
    // }
}
