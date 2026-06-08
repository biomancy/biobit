use biobit_core_py::loc::IntoPyInterval;
use biobit_core_py::loc::IntoPyOrientation;
use biobit_core_py::pickle;
use biobit_reat_rs::selection::RequiredSites;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::OptionExt;
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass(from_py_object, eq, name = "RequiredSites")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Debug, Default, Dissolve, From, Into)]
pub struct PyRequiredSites {
    pub rs: RequiredSites<String, u64>,
}

#[pymethods]
impl PyRequiredSites {
    #[new]
    #[pyo3(signature = (required = None))]
    pub fn new(
        required: Option<Vec<(String, IntoPyOrientation, Vec<IntoPyInterval>)>>,
        py: Python,
    ) -> eyre::Result<Self> {
        let required = required.unwrap_or_default();
        let required = required
            .into_iter()
            .map(|(seqid, orientation, intervals)| {
                let intervals = intervals
                    .into_iter()
                    .map(|interval| interval.0.borrow(py).rs.cast::<u64>())
                    .collect::<Option<Vec<_>>>()
                    .ok_or_eyre("Failed to cast required site interval to u64")?;
                Ok((seqid, orientation.0.0, intervals))
            })
            .collect::<eyre::Result<Vec<_>>>()?;

        Ok(Self {
            rs: RequiredSites::new(required),
        })
    }

    #[getter]
    pub fn len(&self) -> usize {
        self.rs.index().values().map(|sites| sites.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
}
