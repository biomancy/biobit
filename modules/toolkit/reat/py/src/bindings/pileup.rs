use biobit_core_py::loc::PyInterval;
use biobit_core_py::pickle;
use biobit_reat_rs::dna::Reference;
use biobit_reat_rs::pileup::{Pileup, SparsePileup};
use biobit_reat_rs::task::TaskPileup;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::ensure;
use pyo3::PyTypeInfo;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

pub fn interval_to_py(interval: biobit_core_rs::loc::Interval<u64>) -> PyResult<PyInterval> {
    interval.cast::<i64>().map(Into::into).ok_or_else(|| {
        PyValueError::new_err(format!(
            "interval {interval} does not fit into Python interval coordinates"
        ))
    })
}

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

    pub fn a(&self) -> Vec<u32> {
        self.rs.a().to_vec()
    }

    pub fn c(&self) -> Vec<u32> {
        self.rs.c().to_vec()
    }

    pub fn g(&self) -> Vec<u32> {
        self.rs.g().to_vec()
    }

    pub fn t(&self) -> Vec<u32> {
        self.rs.t().to_vec()
    }

    pub fn n(&self) -> Vec<u32> {
        self.rs.n().to_vec()
    }

    pub fn deletion(&self) -> Vec<u32> {
        self.rs.deletion().to_vec()
    }

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
    pub rs: SparsePileup<u64, u32>,
}

#[pymethods]
impl PySparsePileup {
    #[new]
    pub fn new(positions: Vec<u64>, counts: PyPileup) -> eyre::Result<Self> {
        Ok(Self {
            rs: SparsePileup::new(positions, counts.rs)?,
        })
    }

    pub fn positions(&self) -> Vec<u64> {
        self.rs.positions().to_vec()
    }

    pub fn counts(&self) -> PyPileup {
        self.rs.counts().clone().into()
    }

    #[getter]
    pub fn interval(&self) -> PyResult<PyInterval> {
        interval_to_py(self.rs.interval())
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

#[pyclass(from_py_object, eq, name = "TaskPileup")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Debug, Dissolve, From, Into)]
pub struct PyTaskPileup {
    pub rs: TaskPileup<u64, u32>,
}

#[pymethods]
impl PyTaskPileup {
    #[new]
    pub fn new(pileup: PySparsePileup, reference: Vec<String>) -> eyre::Result<Self> {
        let mut refseq = Vec::with_capacity(reference.len());
        for symbol in reference.iter() {
            ensure!(
                symbol.len() == 1,
                "reference symbol must contain exactly one character"
            );
            refseq.push(match symbol.as_bytes()[0] {
                b'A' | b'a' => Reference::A,
                b'C' | b'c' => Reference::C,
                b'G' | b'g' => Reference::G,
                b'T' | b't' => Reference::T,
                b'N' | b'n' => Reference::N,
                _ => eyre::bail!("unsupported reference symbol: {symbol}"),
            });
        }
        Ok(Self {
            rs: TaskPileup::new(pileup.rs, refseq)?,
        })
    }

    pub fn pileup(&self) -> PySparsePileup {
        self.rs.pileup().clone().into()
    }

    pub fn reference(&self) -> Vec<String> {
        self.rs
            .reference()
            .iter()
            .map(|reference| reference.symbol().to_string())
            .collect()
    }

    #[getter]
    pub fn interval(&self) -> PyResult<PyInterval> {
        interval_to_py(self.rs.interval())
    }

    pub fn len(&self) -> usize {
        self.rs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rs.is_empty()
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
