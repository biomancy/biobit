use biobit_core_py::loc::PyOrientation;
use biobit_core_py::pickle;
use biobit_reat_rs::SelectedPileup;
use biobit_reat_rs::pileup::SparsePileup;
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::pileup::PySparsePileup;

type SelectedPileupReduce = (Py<PyAny>, (Py<PyAny>, Vec<u8>));

#[pyclass(name = "SelectedPileup")]
pub struct PySelectedPileup {
    tag: Py<PyAny>,
    pileups: Vec<Py<PySparsePileup>>,
}

impl PySelectedPileup {
    pub fn new(py: Python, tag: Py<PyAny>, pileups: Vec<PySparsePileup>) -> PyResult<Self> {
        let pileups = pileups
            .into_iter()
            .map(|pileup| Py::new(py, pileup))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self { tag, pileups })
    }

    pub fn from_rs(
        py: Python,
        samples: &[Py<PyAny>],
        selected: SelectedPileup<String, u64, u32, usize>,
    ) -> PyResult<Self> {
        let tag = samples[selected.tag].clone_ref(py);
        let pileups = selected
            .pileups
            .into_values()
            .map(PySparsePileup::from)
            .collect::<Vec<_>>();
        Self::new(py, tag, pileups)
    }
}

#[pymethods]
impl PySelectedPileup {
    #[new]
    pub fn py_new(py: Python, tag: Py<PyAny>, pileups: Vec<PySparsePileup>) -> PyResult<Self> {
        Self::new(py, tag, pileups)
    }

    #[getter]
    pub fn tag(&self, py: Python) -> Py<PyAny> {
        self.tag.clone_ref(py)
    }

    pub fn pileups(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for pileup in &self.pileups {
            let borrowed = pileup.borrow(py);
            dict.set_item(
                (
                    borrowed.rs.seqid.clone(),
                    PyOrientation(borrowed.rs.orientation),
                ),
                pileup.clone_ref(py),
            )?;
        }
        Ok(dict.unbind())
    }

    pub fn len(&self) -> usize {
        self.pileups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pileups.is_empty()
    }

    #[staticmethod]
    pub fn _from_pickle(py: Python, tag: Py<PyAny>, state: &Bound<PyBytes>) -> PyResult<Self> {
        let pileups: Vec<SparsePileup<String, u64, u32>> = pickle::from_bytes(state.as_bytes())?;
        let pileups = pileups
            .into_iter()
            .map(PySparsePileup::from)
            .collect::<Vec<_>>();
        Self::new(py, tag, pileups)
    }

    pub fn __reduce__(&self, py: Python) -> PyResult<SelectedPileupReduce> {
        let pileups = self
            .pileups
            .iter()
            .map(|pileup| pileup.borrow(py).rs.clone())
            .collect::<Vec<_>>();
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (self.tag.clone_ref(py), pickle::to_bytes(&pileups)),
        ))
    }
}
