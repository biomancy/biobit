use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::iter::zip;

use derive_getters::{Dissolve, Getters};
use derive_more::{From, Into};
use eyre::eyre;
use itertools::{chain, Itertools};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use biobit_core_py::loc::{IntervalOp, IntoPyInterval, PyInterval};
use biobit_repeto_rs::repeats::InvSegment;

#[pyclass(eq, ord, name = "InvSegment")]
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Getters, Dissolve,
)]
pub struct PyInvSegment {
    rs: InvSegment<i64>,
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

    pub fn __getnewargs__(&self) -> PyResult<(PyInterval, PyInterval)> {
        Ok(((*self.rs.left()).into(), (*self.rs.right()).into()))
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[pyclass(name = "InvRepeat")]
#[derive(Debug, Clone, From, Into, Dissolve)]
pub struct PyInvRepeat {
    pub segments: Vec<Py<PyInvSegment>>,
}

#[pymethods]
impl PyInvRepeat {
    #[new]
    pub fn new(segments: Vec<Py<PyInvSegment>>, py: Python) -> PyResult<Self> {
        if segments.is_empty() {
            Err(eyre!("Inverted repeat must have at least one segment"))?
        }

        // Segments shouldn't overlap
        for (prev, nxt) in segments.iter().tuple_windows() {
            let (p, n) = (prev.borrow(py), nxt.borrow(py));
            if p.rs.left().end() > n.rs.left().start() || p.rs.right().start() < n.rs.right().end()
            {
                Err(eyre!("Segments must be ordered from outer to inner and must not overlap: {:?} vs {:?}", p.rs, n.rs))?
            }
        }

        Ok(Self { segments })
    }

    #[getter]
    pub fn segments(&self, py: Python) -> Vec<Py<PyInvSegment>> {
        self.segments.iter().map(|x| x.clone_ref(py)).collect()
    }

    pub fn seqlen(&self, py: Python) -> i64 {
        self.segments.iter().map(|x| x.borrow(py).rs.seqlen()).sum()
    }

    pub fn inner_gap(&self, py: Python) -> i64 {
        self.segments.last().unwrap().borrow(py).rs.inner_gap()
    }

    pub fn left_brange(&self, py: Python) -> PyInterval {
        let start = self.segments[0].borrow(py).rs.left().start();
        let end = self.segments.last().unwrap().borrow(py).rs.left().end();
        PyInterval::new(start, end).unwrap()
    }

    pub fn right_brange(&self, py: Python) -> PyInterval {
        let start = self.segments.last().unwrap().borrow(py).rs.right().start();
        let end = self.segments[0].borrow(py).rs.right().end();
        PyInterval::new(start, end).unwrap()
    }

    pub fn brange(&self, py: Python) -> PyInterval {
        self.segments[0].borrow(py).rs.brange().into()
    }

    pub fn shift<'a>(mut slf: PyRefMut<'a, Self>, py: Python, shift: i64) -> PyRefMut<'a, Self> {
        for x in &mut slf.segments {
            x.borrow_mut(py).rs.shift(shift);
        }
        slf
    }

    pub fn seqranges(&self, py: Python) -> Vec<PyInterval> {
        chain(
            self.segments
                .iter()
                .map(|x| (*x.borrow(py).rs.left()).into()),
            self.segments
                .iter()
                .rev()
                .map(|x| (*x.borrow(py).rs.right()).into()),
        )
        .collect()
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(
        signature = (contig, *args, name = ".", score = 0, strand = ".", color = "0,0,0"),
        text_signature = None
    )]
    pub fn to_bed12(
        &self,
        py: Python,
        contig: &str,
        args: Bound<PyTuple>,
        name: &str,
        score: u16,
        strand: &str,
        color: &str,
    ) -> PyResult<String> {
        if !args.is_empty() {
            return Err(eyre!(
                "to_bed12 doesn't support positional arguments except 'contig'"
            ))?;
        } else if score > 1000 {
            return Err(eyre!("Score must be from 0 to 1000"))?;
        }

        let range = self.brange(py);
        let (block_sizes, block_starts): (Vec<usize>, Vec<i64>) = self
            .seqranges(py)
            .into_iter()
            .map(|x| (x.rs.len() as usize, x.rs.start() - range.start()))
            .unzip();
        let block_sizes = block_sizes.into_iter().join(",");
        let block_starts = block_starts.into_iter().join(",");

        Ok(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            contig,
            range.start(),
            range.end(),
            name,
            score,
            strand,
            range.start(),
            range.end(),
            color,
            self.segments.len() * 2,
            block_sizes,
            block_starts
        ))
    }

    pub fn __len__(&self, py: Python) -> usize {
        self.segments.iter().map(|x| x.borrow(py).__len__()).sum()
    }

    pub fn __eq__(&self, other: &Self, py: Python) -> bool {
        if self.segments.len() != other.segments.len() {
            return false;
        }

        let mut alleq = true;
        for (a, b) in zip(&self.segments, &other.segments) {
            let (a, b) = (a.borrow(py), &b.borrow(py));

            if a.rs != b.rs {
                alleq = false;
                break;
            }
        }
        alleq
    }

    pub fn __getnewargs__(&self, py: Python) -> (Vec<Py<PyInvSegment>>,) {
        (self.segments.iter().map(|x| x.clone_ref(py)).collect(),)
    }

    pub fn __hash__(&self, py: Python) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        for s in &self.segments {
            s.borrow(py).hash(&mut hasher);
        }
        Ok(hasher.finish())
    }
}
