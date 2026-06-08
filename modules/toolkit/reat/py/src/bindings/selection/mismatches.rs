use biobit_core_py::pickle;
use biobit_reat_rs::selection::Mismatches;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass(from_py_object, eq, name = "Mismatches")]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Debug, Default, Dissolve, From, Into)]
pub struct PyMismatches {
    pub rs: Mismatches<u32>,
}

#[pymethods]
impl PyMismatches {
    #[new]
    #[pyo3(signature = (minmismatches = 1, minfreq = 0.0))]
    pub fn new(minmismatches: u32, minfreq: f32) -> eyre::Result<Self> {
        Ok(Self {
            rs: Mismatches::new(minmismatches, minfreq)?,
        })
    }

    #[getter]
    pub fn minmismatches(&self) -> u32 {
        self.rs.minmismatches()
    }

    #[getter]
    pub fn minfreq(&self) -> f32 {
        self.rs.minfreq()
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
