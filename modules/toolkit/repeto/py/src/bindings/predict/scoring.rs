use derive_more::{From, Into};
use itertools::Itertools;
use pyo3::prelude::*;

use biobit_repeto_rs::predict::Score;
pub use biobit_repeto_rs::predict::Scoring;

#[pyclass(eq, ord, name = "Scoring")]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct PyScoring {
    rs: Scoring<i32>,
}

#[pymethods]
impl PyScoring {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[getter]
    pub fn gap_open(&self) -> i32 {
        self.rs.gap_open
    }

    #[setter]
    pub fn set_gap_open(&mut self, value: i32) {
        self.rs.gap_open = value;
    }

    #[getter]
    pub fn gap_extend(&self) -> i32 {
        self.rs.gap_extend
    }

    #[setter]
    pub fn set_gap_extend(&mut self, value: i32) {
        self.rs.gap_extend = value;
    }

    #[getter]
    pub fn complementary(&self) -> i32 {
        self.rs.complementary
    }

    #[setter]
    pub fn set_complementary(&mut self, value: i32) {
        self.rs.complementary = value;
    }

    #[getter]
    pub fn mismatch(&self) -> i32 {
        self.rs.mismatch
    }

    #[setter]
    pub fn set_mismatch(&mut self, value: i32) {
        self.rs.mismatch = value;
    }

    pub fn __getstate__(&self) -> (i32, i32, i32, i32) {
        (
            self.rs.complementary,
            self.rs.mismatch,
            self.rs.gap_open,
            self.rs.gap_extend,
        )
    }

    pub fn __setstate__(&mut self, state: (i32, i32, i32, i32)) {
        self.rs.complementary = state.0;
        self.rs.mismatch = state.1;
        self.rs.gap_open = state.2;
        self.rs.gap_extend = state.3;
    }
}

impl<S: Score + TryInto<i32>> IntoPy<PyScoring> for Scoring<S> {
    fn into_py(self, _: Python) -> PyScoring {
        let dissolved = self.dissolve();
        let dissolved = [dissolved.0, dissolved.1, dissolved.2, dissolved.3];

        let (complementary, mismatch, gap_open, gap_extend) = dissolved
            .into_iter()
            .map(|val| match val.try_into() {
                Ok(val) => val,
                Err(_) => panic!("Cannot convert {:?} to i32", val),
            })
            .collect_tuple::<(_, _, _, _)>()
            .unwrap();

        let rs = Scoring::<i32> {
            complementary,
            mismatch,
            gap_open,
            gap_extend,
        };
        PyScoring { rs }
    }
}
