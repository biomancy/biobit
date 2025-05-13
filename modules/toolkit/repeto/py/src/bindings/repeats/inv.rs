use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::{OptionExt, Result, eyre};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

use biobit_core_py::loc::{
    IntervalOp, IntoPyInterval, IntoPyOrientation, PyInterval, PyOrientation,
};
use biobit_core_py::pickle;
use biobit_io_py::bed::{Bed12, PyBed12};
use biobit_repeto_rs::repeats::{InvRepeat, InvSegment};
use bitcode::{Decode, Encode};
use pyo3::PyTypeInfo;

#[pyclass(eq, ord, name = "InvSegment")]
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Dissolve, Encode, Decode,
)]
pub struct PyInvSegment {
    pub rs: InvSegment<i64>,
}

#[pymethods]
impl PyInvSegment {
    #[new]
    pub fn new(py: Python, left: IntoPyInterval, right: IntoPyInterval) -> PyResult<Self> {
        {
            let left = left.0.borrow(py).rs;
            let right = right.0.borrow(py).rs;

            let rs = InvSegment::new(left, right)?;
            Ok(PyInvSegment { rs })
        }
    }

    #[getter]
    pub fn left(&self) -> PyInterval {
        (*self.rs.left()).into()
    }

    #[getter]
    pub fn right(&self) -> PyInterval {
        (*self.rs.right()).into()
    }

    pub fn brange(&self) -> PyInterval {
        self.rs.brange().into()
    }

    pub fn inner_gap(&self) -> i64 {
        self.rs.inner_gap()
    }

    pub fn shift(mut slf: PyRefMut<Self>, shift: i64) -> PyRefMut<Self> {
        slf.rs.shift(shift);
        slf
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.rs)
    }

    pub fn __str__(&self) -> String {
        format!("{}", self.rs)
    }

    pub fn __len__(&self) -> usize {
        self.rs.len() as usize
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[staticmethod]
    pub fn _from_pickle(state: &Bound<PyBytes>) -> PyResult<Self> {
        pickle::from_bytes(state.as_bytes()).map_err(|e| e.into())
    }

    pub fn __reduce__(&self, py: Python) -> Result<(PyObject, (Vec<u8>,))> {
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (pickle::to_bytes(self),),
        ))
    }
}

#[pyclass(name = "InvRepeat")]
#[derive(Debug, Clone, From, Into, Decode, Encode, Dissolve)]
pub struct PyInvRepeat {
    pub rs: InvRepeat<i64>,
}

#[pymethods]
impl PyInvRepeat {
    #[new]
    pub fn new(segments: Vec<PyInvSegment>) -> Result<Self> {
        let segments = segments.into_iter().map(|x| x.rs).collect::<Vec<_>>();

        InvRepeat::new(segments).map(Self::from)
    }

    #[getter]
    pub fn segments(&self) -> Vec<PyInvSegment> {
        self.rs
            .segments()
            .iter()
            .map(|x| PyInvSegment::from(*x))
            .collect()
    }

    pub fn seqlen(&self) -> i64 {
        self.rs.seqlen()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i64 {
        self.rs.len()
    }

    pub fn inner_gap(&self) -> i64 {
        self.rs.inner_gap()
    }

    pub fn left_brange(&self) -> PyInterval {
        self.rs.left_brange().into()
    }

    pub fn right_brange(&self) -> PyInterval {
        self.rs.right_brange().into()
    }

    pub fn brange(&self) -> PyInterval {
        self.rs.brange().into()
    }

    pub fn shift(mut slf: PyRefMut<Self>, shift: i64) -> PyRefMut<Self> {
        slf.rs.shift(shift);
        slf
    }

    pub fn seqranges(&self) -> Vec<PyInterval> {
        self.rs.seqranges().map(|x| (*x).into()).collect()
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(
        signature = (seqid, *args, name = ".", score = 0, orientation = IntoPyOrientation::from(PyOrientation::Dual), rgb = (0, 0, 0)),
        text_signature = None
    )]
    pub fn to_bed12(
        &self,
        seqid: &str,
        args: Bound<PyTuple>,
        name: &str,
        score: u16,
        orientation: IntoPyOrientation,
        rgb: (u8, u8, u8),
    ) -> Result<PyBed12> {
        if !args.is_empty() {
            return Err(eyre!(
                "to_bed12 doesn't support positional arguments except 'seqid'"
            ))?;
        }

        let brange = self.brange().rs;
        let blocks = self
            .seqranges()
            .iter()
            .map(|x| x.rs.shifted(-brange.start()).cast())
            .collect::<Option<Vec<_>>>();

        let brange = brange
            .cast()
            .ok_or_eyre("Inverted repeats coordinates must be strictly positive.")?;
        let blocks =
            blocks.ok_or_eyre("Inverted repeats coordinates must be strictly positive.")?;

        Bed12::new(
            seqid.to_owned(),
            brange,
            name.to_owned(),
            score,
            orientation.0.0,
            brange,
            rgb,
            blocks,
        )
        .map(PyBed12::from)
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.rs == other.rs
    }

    pub fn __hash__(&self) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        self.rs.hash(&mut hasher);
        Ok(hasher.finish())
    }

    pub fn __len__(&self) -> usize {
        self.rs.len() as usize
    }

    #[staticmethod]
    pub fn _from_pickle(state: &Bound<PyBytes>) -> PyResult<Self> {
        pickle::from_bytes(state.as_bytes()).map_err(|e| e.into())
    }

    pub fn __reduce__(&self, py: Python) -> Result<(PyObject, (Vec<u8>,))> {
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (pickle::to_bytes(self),),
        ))
    }
}
