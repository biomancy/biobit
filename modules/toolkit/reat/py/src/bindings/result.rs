use biobit_core_py::loc::PyOrientation;
use biobit_core_py::pickle;
use biobit_reat_rs::SamplePileup;
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::PyTaskPileup;

#[pyclass(name = "SamplePileup")]
#[repr(transparent)]
pub struct PySamplePileup {
    rs: SamplePileup<String, u64, u32, Py<PyAny>>,
}

impl PySamplePileup {
    pub fn new(tag: Py<PyAny>, rs: SamplePileup<String, u64, u32, usize>) -> Self {
        let rs = rs.retag(tag);
        Self { rs }
    }
}

#[pymethods]
impl PySamplePileup {
    #[getter]
    pub fn tag(&self, py: Python) -> Py<PyAny> {
        self.rs.tag.clone_ref(py)
    }

    pub fn pileups(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (key, value) in self.rs.pileups.iter() {
            let (seqid, orientation) = key;
            dict.set_item(
                (seqid.clone(), PyOrientation(*orientation)),
                PyTaskPileup { rs: value.clone() },
            )?;
        }
        Ok(dict.unbind())
    }

    pub fn len(&self) -> usize {
        self.rs.pileups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rs.pileups.is_empty()
    }

    #[staticmethod]
    pub fn _from_pickle(tag: Py<PyAny>, state: &Bound<PyBytes>) -> PyResult<Self> {
        let pileups = pickle::from_bytes(state.as_bytes())?;
        let rs = SamplePileup { tag, pileups };
        Ok(Self { rs })
    }

    #[allow(clippy::type_complexity)]
    pub fn __reduce__(&self, py: Python) -> PyResult<(Py<PyAny>, (Py<PyAny>, Vec<u8>))> {
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (
                self.rs.tag.clone_ref(py),
                pickle::to_bytes(&self.rs.pileups),
            ),
        ))
    }
}
