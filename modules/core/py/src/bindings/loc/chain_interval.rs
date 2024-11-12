use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyList, PySequence};

use super::interval::IntoPyInterval;
use crate::loc::PyInterval;
use biobit_core_rs::loc::ChainInterval;

#[pyclass]
#[repr(transparent)]
#[derive(Debug, Into, From)]
pub struct IntoPyChainInterval {
    pub py: PyChainInterval,
}

impl<'py> FromPyObject<'py> for IntoPyChainInterval {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let chain = if obj.is_instance_of::<PyChainInterval>() {
            obj.downcast::<PyChainInterval>()?.borrow().clone()
        } else {
            let seq = obj.downcast::<PySequence>().map_err(|err| {
                PyValueError::new_err(format!("Invalid ChainInterval interval: {}", err))
            })?;

            let mut links = Vec::with_capacity(seq.len()?);
            for it in seq.iter()? {
                let link = IntoPyInterval::extract_bound(&it?)?;
                links.push(link.0.borrow(obj.py()).rs);
            }

            ChainInterval::try_from_iter(links.into_iter())?.into()
        };
        Ok(IntoPyChainInterval { py: chain })
    }
}

#[pyclass(name = "ChainInterval")]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, From, Into)]
pub struct PyChainInterval {
    pub rs: ChainInterval<i64>,
}

#[pymethods]
impl PyChainInterval {
    #[new]
    pub fn new(source: IntoPyChainInterval) -> PyResult<Self> {
        Ok(source.into())
    }

    pub fn __getnewargs__(&self) -> (Vec<PyInterval>,) {
        (self
            .rs
            .links()
            .iter()
            .map(|x| PyInterval::from(*x))
            .collect(),)
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let links = self
            .rs
            .links()
            .iter()
            .map(|x| PyInterval::from(*x).into_py(py));

        let result = PyList::new_bound(py, links);
        PyIterator::from_bound_object(result.as_any())
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __str__(&self) -> String {
        self.rs.to_string()
    }

    fn __richcmp__(&self, other: IntoPyChainInterval, op: CompareOp) -> bool {
        println!("{} vs {}", self.rs, other.py.rs);
        match op {
            CompareOp::Eq => self.rs == other.py.rs,
            CompareOp::Ne => self.rs != other.py.rs,
            CompareOp::Lt => self.rs < other.py.rs,
            CompareOp::Le => self.rs <= other.py.rs,
            CompareOp::Gt => self.rs > other.py.rs,
            CompareOp::Ge => self.rs >= other.py.rs,
        }
    }
}
