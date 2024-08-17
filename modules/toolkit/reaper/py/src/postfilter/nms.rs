use biobit_core_py::loc::{IntoPyOrientation, Orientation};
use biobit_reaper_rs::postfilter::NMS;
use derive_getters::Dissolve;
use derive_more::{Constructor, From, Into};
use pyo3::prelude::*;

#[pyclass(eq, name = "NMS")]
#[derive(Clone, PartialEq, Debug, Constructor, Dissolve, From, Into)]
pub struct PyNMS {
    rs: NMS<usize, f32>,
}

#[pymethods]
impl PyNMS {
    #[new]
    pub fn __new__() -> Self {
        PyNMS::new(NMS::new())
    }

    pub fn set_fecutoff(mut slf: PyRefMut<Self>, fecutoff: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_fecutoff(fecutoff)?;
        Ok(slf)
    }

    pub fn set_group_within(
        mut slf: PyRefMut<Self>,
        group_within: usize,
    ) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_group_within(group_within)?;
        Ok(slf)
    }

    pub fn set_slopfrac(mut slf: PyRefMut<Self>, slopfrac: f32) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_slopfrac(slopfrac)?;
        Ok(slf)
    }

    pub fn set_sloplim(
        mut slf: PyRefMut<Self>,
        minslop: usize,
        maxslop: usize,
    ) -> PyResult<PyRefMut<Self>> {
        slf.rs.set_sloplim(minslop, maxslop)?;
        Ok(slf)
    }

    pub fn set_boundaries(
        mut slf: PyRefMut<Self>,
        orientation: IntoPyOrientation,
        boundaries: Vec<usize>,
    ) -> PyResult<PyRefMut<Self>> {
        slf.rs
            .set_boundaries(orientation.0 .0, boundaries.into_iter().collect());
        Ok(slf)
    }

    pub fn __getstate__(
        &self,
    ) -> (
        f32,
        usize,
        f32,
        (usize, usize),
        (Vec<usize>, Vec<usize>, Vec<usize>),
    ) {
        (
            *self.rs.fecutoff(),
            *self.rs.group_within(),
            *self.rs.slopfrac(),
            *self.rs.sloplim(),
            self.rs.boundaries().clone().dissolve(),
        )
    }

    pub fn __setstate__(
        &mut self,
        state: (
            f32,
            usize,
            f32,
            (usize, usize),
            (Vec<usize>, Vec<usize>, Vec<usize>),
        ),
    ) {
        self.rs.set_fecutoff(state.0).unwrap();
        self.rs.set_group_within(state.1).unwrap();
        self.rs.set_slopfrac(state.2).unwrap();
        self.rs.set_sloplim(state.3 .0, state.3 .1).unwrap();

        self.rs
            .set_boundaries_trusted(Orientation::Forward, state.4 .0);
        self.rs
            .set_boundaries_trusted(Orientation::Reverse, state.4 .1);
        self.rs
            .set_boundaries_trusted(Orientation::Dual, state.4 .2);
    }
}
