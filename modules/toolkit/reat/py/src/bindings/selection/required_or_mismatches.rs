use biobit_core_py::pickle;
use biobit_reat_rs::selection::RequiredOrMismatches;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use super::{PyMismatches, PyRequiredSites};

#[pyclass(from_py_object, eq, name = "RequiredOrMismatches")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Debug, Default, Dissolve, From, Into)]
pub struct PyRequiredOrMismatches {
    pub rs: RequiredOrMismatches<String, u64, u32>,
}

#[pymethods]
impl PyRequiredOrMismatches {
    #[new]
    #[pyo3(signature = (required = None, mismatches = None))]
    pub fn new(required: Option<PyRequiredSites>, mismatches: Option<PyMismatches>) -> Self {
        Self {
            rs: RequiredOrMismatches::new(
                required.unwrap_or_default().rs,
                mismatches.unwrap_or_default().rs,
            ),
        }
    }

    #[getter]
    pub fn required(&self) -> PyRequiredSites {
        self.rs.required().clone().into()
    }

    #[getter]
    pub fn mismatches(&self) -> PyMismatches {
        (*self.rs.mismatches()).into()
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
