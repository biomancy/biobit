use crate::loc::{IntoPyChainInterval, IntoPyInterval, PyChainInterval, PyInterval};
pub use biobit_core_rs::loc::mapping::ChainMap;
use biobit_core_rs::loc::mapping::Mapping;
use biobit_core_rs::loc::{ChainInterval, Interval};
use bitcode::{Decode, Encode};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::prelude::*;

#[pyclass(frozen, eq, hash, name = "ChainMap")]
#[repr(transparent)]
#[derive(
    Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve, From, Into,
)]
pub struct PyChainMap {
    pub rs: ChainMap<i64>,
}

#[pymethods]
impl PyChainMap {
    #[new]
    pub fn new(chain: IntoPyChainInterval) -> Self {
        ChainMap::new(chain.py.rs).into()
    }

    pub fn invmap_interval(&self, py: Python, interval: IntoPyInterval) -> Option<PyChainInterval> {
        let interval: Interval<_> = interval.0.bind(py).borrow().rs;
        match self.rs.invmap_interval(&interval, Default::default()) {
            Mapping::Complete(x) | Mapping::Truncated(x) => Some(PyChainInterval::from(x)),
            Mapping::None => None,
        }
    }

    pub fn map_interval(&self, py: Python, interval: IntoPyInterval) -> Option<PyInterval> {
        let interval: Interval<_> = interval.0.bind(py).borrow().rs;
        match self.rs.map_interval(&interval) {
            Mapping::Complete(x) | Mapping::Truncated(x) => Some(PyInterval::from(x)),
            Mapping::None => None,
        }
    }

    pub fn __getnewargs__(&self) -> (PyChainInterval,) {
        (ChainInterval::from(self.rs.clone()).into(),)
    }
}
