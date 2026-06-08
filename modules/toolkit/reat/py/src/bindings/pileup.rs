use biobit_core_py::loc::{IntoPyOrientation, PyInterval, PyOrientation};
use biobit_core_py::pickle;
use biobit_reat_rs::pileup::{Pileup, SparsePileup};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass(from_py_object, eq, name = "Pileup")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Debug, Default, Dissolve, From, Into)]
pub struct PyPileup {
    pub rs: Pileup<u32>,
}

#[pymethods]
impl PyPileup {
    #[new]
    pub fn new(
        a: Vec<u32>,
        c: Vec<u32>,
        g: Vec<u32>,
        t: Vec<u32>,
        n: Vec<u32>,
        deletion: Vec<u32>,
    ) -> eyre::Result<Self> {
        Ok(Self {
            rs: Pileup::new(a, c, g, t, n, deletion)?,
        })
    }

    #[staticmethod]
    pub fn zeros(len: usize) -> Self {
        Self {
            rs: Pileup::zeros(len),
        }
    }

    pub fn len(&self) -> usize {
        self.rs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rs.is_empty()
    }

    #[getter]
    pub fn a(&self) -> Vec<u32> {
        self.rs.a().to_vec()
    }

    #[getter]
    pub fn c(&self) -> Vec<u32> {
        self.rs.c().to_vec()
    }

    #[getter]
    pub fn g(&self) -> Vec<u32> {
        self.rs.g().to_vec()
    }

    #[getter]
    pub fn t(&self) -> Vec<u32> {
        self.rs.t().to_vec()
    }

    #[getter]
    pub fn n(&self) -> Vec<u32> {
        self.rs.n().to_vec()
    }

    #[getter]
    pub fn deletion(&self) -> Vec<u32> {
        self.rs.deletion().to_vec()
    }

    #[getter]
    pub fn coverage(&self) -> Vec<u32> {
        self.rs
            .iter()
            .map(|site| site.coverage())
            .collect::<Vec<_>>()
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

#[pyclass(from_py_object, eq, name = "SparsePileup")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Debug, Dissolve, From, Into)]
pub struct PySparsePileup {
    pub rs: SparsePileup<String, usize, u32>,
}

#[pymethods]
impl PySparsePileup {
    #[new]
    pub fn new(
        seqid: String,
        orientation: IntoPyOrientation,
        positions: Vec<usize>,
        counts: PyPileup,
    ) -> eyre::Result<Self> {
        Ok(Self {
            rs: SparsePileup::new(seqid, orientation.0.0, positions, counts.rs)?,
        })
    }

    #[getter]
    pub fn seqid(&self) -> &str {
        &self.rs.seqid
    }

    #[getter]
    pub fn orientation(&self) -> PyOrientation {
        self.rs.orientation.into()
    }

    #[getter]
    pub fn positions(&self) -> Vec<usize> {
        self.rs.positions().to_vec()
    }

    #[getter]
    pub fn counts(&self) -> PyPileup {
        self.rs.counts().clone().into()
    }

    #[getter]
    pub fn interval(&self) -> PyInterval {
        self.rs.interval().cast::<i64>().unwrap().into()
    }

    pub fn len(&self) -> usize {
        self.rs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rs.len() == 0
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
