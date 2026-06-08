use biobit_core_py::loc::{IntoPyInterval, PyInterval};
use biobit_core_py::pickle;
use biobit_core_py::utils::type_hint_class_getitem;
use biobit_reat_rs::task::Task;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::OptionExt;
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};

#[pyclass(from_py_object, eq, name = "Task")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Debug, Dissolve, From, Into)]
pub struct PyTask {
    pub rs: Task<String, u64>,
}

#[pymethods]
impl PyTask {
    #[new]
    pub fn new(seqid: String, intervals: Vec<IntoPyInterval>, py: Python) -> eyre::Result<Self> {
        let intervals = intervals
            .into_iter()
            .map(|interval| interval.0.borrow(py).rs.cast::<u64>())
            .collect::<Option<Vec<_>>>()
            .ok_or_eyre("Failed to cast task interval to u64")?;

        Ok(Self {
            rs: Task::new(seqid, intervals)?,
        })
    }

    #[staticmethod]
    pub fn from_intervals(
        intervals: Vec<(String, IntoPyInterval)>,
        max_task_size: u64,
        py: Python,
    ) -> eyre::Result<Vec<Self>> {
        let intervals = intervals
            .into_iter()
            .map(|(seqid, interval)| {
                let interval = interval
                    .0
                    .borrow(py)
                    .rs
                    .cast::<u64>()
                    .ok_or_eyre("Failed to cast task interval to u64")?;
                Ok((seqid, interval))
            })
            .collect::<eyre::Result<Vec<_>>>()?;
        Ok(Task::from_intervals(intervals, max_task_size)?
            .into_iter()
            .map(Self::from)
            .collect())
    }

    #[getter]
    pub fn seqid(&self) -> &str {
        self.rs.seqid()
    }

    #[getter]
    pub fn envelope(&self) -> PyInterval {
        self.rs.envelope().cast::<i64>().unwrap().into()
    }

    #[getter]
    pub fn intervals(&self) -> Vec<PyInterval> {
        self.rs
            .intervals()
            .iter()
            .map(|interval| interval.cast::<i64>().unwrap().into())
            .collect()
    }

    #[staticmethod]
    pub fn _from_pickle(state: &Bound<PyBytes>) -> PyResult<Self> {
        pickle::from_bytes(state.as_bytes())
            .map(|rs| Self { rs })
            .map_err(|err| err.into())
    }

    pub fn __reduce__(&self, py: Python) -> eyre::Result<(Py<PyAny>, (Vec<u8>,))> {
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (pickle::to_bytes(&self.rs),),
        ))
    }

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: Py<PyAny>) -> PyResult<Py<PyAny>> {
        type_hint_class_getitem(cls, args)
    }
}
