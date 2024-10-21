use derive_more::{From, Into};
use pyo3::prelude::*;

use biobit_core_py::loc::{IntoPySegment, PySegment};
use biobit_repeto_rs::predict::Filter;

#[pyclass(eq, ord, name = "Filter")]
#[derive(Clone, PartialEq, PartialOrd, Debug, Hash, Default, From, Into)]
pub struct PyFilter {
    rs: Filter<i32>,
}

#[pymethods]
impl PyFilter {
    #[new]
    pub fn new() -> Self {
        PyFilter {
            rs: Filter::default(),
        }
    }

    pub fn set_min_score(mut slf: PyRefMut<Self>, min_score: i32) -> PyRefMut<Self> {
        slf.rs.set_min_score(min_score);
        slf
    }

    pub fn set_rois<'a>(
        mut slf: PyRefMut<'a, Self>,
        rois: Vec<IntoPySegment>,
        py: Python,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let segments = rois
            .into_iter()
            .map(|x| x.0.borrow(py).rs.try_cast::<usize>())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        slf.rs.set_rois(segments);
        Ok(slf)
    }

    pub fn set_min_roi_overlap(
        mut slf: PyRefMut<Self>,
        total: usize,
        ungapped: usize,
    ) -> PyRefMut<Self> {
        slf.rs.set_min_roi_overlap(total, ungapped);
        slf
    }

    pub fn set_min_matches(
        mut slf: PyRefMut<Self>,
        total: usize,
        ungapped: usize,
    ) -> PyRefMut<Self> {
        slf.rs.set_min_matches(total, ungapped);
        slf
    }

    pub fn __getstate__(&self, py: Python) -> (i32, (usize, usize, usize, usize), Vec<PySegment>) {
        let stats = self.rs.stats();
        let stats = (
            stats.in_roi.total_len,
            stats.in_roi.max_len,
            stats.all.total_len,
            stats.all.max_len,
        );
        let rois = self
            .rs
            .rois()
            .iter()
            .map(|x| x.into_py(py))
            .collect::<Vec<_>>();

        (*self.rs.min_score(), stats, rois)
    }

    pub fn __setstate__(
        mut slf: PyRefMut<Self>,
        state: (i32, (usize, usize, usize, usize), Vec<PySegment>),
    ) -> PyRefMut<Self> {
        slf.rs.set_min_score(state.0);
        slf.rs.set_min_roi_overlap(state.1 .0, state.1 .1);
        slf.rs.set_min_matches(state.1 .2, state.1 .3);
        let rois = state
            .2
            .into_iter()
            .map(|x| x.rs.try_cast::<usize>())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        slf.rs.set_rois(rois);
        slf
    }
}