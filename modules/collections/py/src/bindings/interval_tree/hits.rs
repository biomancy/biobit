use crate::interval_tree::{PyBatchHitSegments, PyHitSegments};
use biobit_collections_rs::interval_tree::{BatchHits, Hits};
use biobit_core_py::loc::{Interval, IntoPyInterval, PyInterval};
use biobit_core_py::pickle;
use biobit_core_py::utils::ByPyPointer;
use derive_getters::Dissolve;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyIterator, PyList};
use pyo3::{pyclass, pymethods, PyObject};

#[pyclass(name = "Hits")]
#[derive(Default, Dissolve)]
pub struct PyHits {
    cache: Option<Hits<'static, i64, PyObject>>,
    intervals: Vec<Interval<i64>>,
    data: Vec<PyObject>,
}

impl PyHits {
    pub fn take<'tree, T>(&mut self) -> Hits<'tree, i64, T> {
        self.cache.take().unwrap_or_default().recycle()
    }

    pub fn reset(&mut self, py: Python, hits: Hits<i64, PyObject>) {
        self.intervals.extend(hits.intervals());
        self.data
            .extend(hits.data().iter().map(|x| x.clone_ref(py)));
        self.cache = Some(hits.recycle());
    }
}

#[pymethods]
impl PyHits {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intervals<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, self.intervals.iter().map(|x| PyInterval::from(*x)))
    }

    pub fn data<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, self.data.iter())
    }

    #[pyo3(signature = (query, into = None))]
    pub fn segment<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        query: Vec<IntoPyInterval>,
        into: Option<Py<PyHitSegments>>,
    ) -> PyResult<Py<PyHitSegments>> {
        let into = match into {
            Some(into) => into,
            None => Py::new(py, PyHitSegments::new())?,
        };
        {
            let mut borrow = into.borrow_mut(py);
            let mut segments = borrow.take();

            // Run the segmentation.
            let query = query.into_iter().map(|x| x.extract_py(py).rs);
            let data = slf.data.iter().map(|x| ByPyPointer::from_ref(x));

            segments.build_from_parts(query, &slf.intervals, data)?;
            borrow.reset(py, segments)?;
        }

        Ok(into)
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
        self.data.clear();
    }

    pub fn __len__(&self) -> usize {
        self.intervals.len()
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::new(
            py,
            self.intervals
                .iter()
                .zip(self.data.iter())
                .map(|(interval, hit)| (PyInterval::from(*interval), hit.clone_ref(py))),
        )?;
        result.try_iter()
    }

    pub fn __eq__(&self, other: &PyHits) -> PyResult<bool> {
        if self.intervals != other.intervals || self.data.len() != other.data.len() {
            return Ok(false);
        }

        Python::with_gil(|py| -> PyResult<bool> {
            for (i, j) in self.data.iter().zip(other.data.iter()) {
                if !i.bind(py).eq(j.bind(py))? {
                    return Ok(false);
                }
            }
            Ok(true)
        })
    }

    pub fn __getstate__<'a>(&self, py: Python<'a>) -> PyResult<(Vec<u8>, Bound<'a, PyList>)> {
        Ok((pickle::to_bytes(&self.intervals), self.data(py)?))
    }

    pub fn __setstate__(&mut self, state: (Bound<PyBytes>, Vec<PyObject>)) -> PyResult<()> {
        self.intervals = pickle::from_bytes(state.0.as_bytes())?;
        self.data = state.1;
        Ok(())
    }
}

#[pyclass(name = "BatchHits")]
#[derive(Default, Dissolve)]
pub struct PyBatchHits {
    cache: Option<BatchHits<'static, i64, PyObject>>,
    intervals: Vec<PyInterval>,
    hits: Vec<PyObject>,
    index: Vec<usize>,
}

impl PyBatchHits {
    pub fn take<'tree>(&mut self) -> BatchHits<'tree, i64, PyObject> {
        self.cache.take().unwrap_or_default().recycle()
    }

    pub fn reset(&mut self, py: Python, hits: BatchHits<i64, PyObject>) {
        self.clear();

        for (intervals, hits) in hits.iter() {
            self.intervals
                .extend(intervals.iter().map(|x| PyInterval::from(*x)));
            self.hits.extend(hits.iter().map(|x| x.clone_ref(py)));
            self.index.push(self.intervals.len())
        }

        self.cache = Some(hits.recycle());
    }
}

#[pymethods]
impl PyBatchHits {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intervals<'a>(&self, py: Python<'a>, i: usize) -> PyResult<Bound<'a, PyList>> {
        if i + 1 >= self.index.len() {
            Err(pyo3::exceptions::PyIndexError::new_err(
                "Index out of bounds",
            ))
        } else {
            PyList::new(
                py,
                self.intervals[self.index[i]..self.index[i + 1]]
                    .iter()
                    .cloned(),
            )
        }
    }

    pub fn data<'a>(&self, py: Python<'a>, i: usize) -> PyResult<Bound<'a, PyList>> {
        if i + 1 >= self.index.len() {
            Err(pyo3::exceptions::PyIndexError::new_err(
                "Index out of bounds",
            ))
        } else {
            PyList::new(py, self.hits[self.index[i]..self.index[i + 1]].iter())
        }
    }

    #[pyo3(signature = (query, into = None))]
    pub fn segment<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        query: Vec<Vec<IntoPyInterval>>,
        into: Option<Py<PyBatchHitSegments>>,
    ) -> PyResult<Py<PyBatchHitSegments>> {
        let into = match into {
            Some(into) => into,
            None => Py::new(py, PyBatchHitSegments::new())?,
        };
        {
            let mut borrow = into.borrow_mut(py);
            let mut segments = borrow.take();

            // Run the segmentation.
            let query = query
                .into_iter()
                .map(|x| x.into_iter().map(|x| x.extract_py(py).rs));
            let intervals = (0..slf.index.len() - 1).map(|i| {
                slf.intervals[slf.index[i]..slf.index[i + 1]]
                    .iter()
                    .map(|x| x.rs)
            });
            let data = (0..slf.index.len() - 1).map(|i| {
                slf.hits[slf.index[i]..slf.index[i + 1]]
                    .iter()
                    .map(|x| ByPyPointer::from_ref(x))
            });

            segments.build_from_parts(query, intervals, data)?;
            borrow.reset(py, segments)?;
        }

        Ok(into)
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
        self.hits.clear();
        self.index.clear();
        self.index.push(0);
    }

    pub fn __len__(&self) -> usize {
        self.index.len() - 1
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::empty(py);
        for i in 0..self.index.len() - 1 {
            result.append((self.intervals(py, i)?, self.data(py, i)?))?
        }
        result.try_iter()
    }

    pub fn __eq__(&self, other: &PyBatchHits) -> PyResult<bool> {
        if self.index != other.index
            || self.intervals != other.intervals
            || self.hits.len() != other.hits.len()
        {
            return Ok(false);
        }

        Python::with_gil(|py| -> PyResult<bool> {
            for (i, j) in self.hits.iter().zip(other.hits.iter()) {
                if !i.bind(py).eq(j.bind(py))? {
                    return Ok(false);
                }
            }
            Ok(true)
        })
    }

    pub fn __getstate__<'a>(
        &self,
        py: Python<'a>,
    ) -> PyResult<(Vec<u8>, Bound<'a, PyList>, Vec<u8>)> {
        Ok((
            pickle::to_bytes(&self.intervals),
            PyList::new(py, self.hits.iter())?,
            pickle::to_bytes(&self.index),
        ))
    }

    pub fn __setstate__(
        &mut self,
        state: (Bound<PyBytes>, Vec<PyObject>, Bound<PyBytes>),
    ) -> PyResult<()> {
        self.intervals = pickle::from_bytes(state.0.as_bytes())?;
        self.hits = state.1;
        self.index = pickle::from_bytes(state.2.as_bytes())?;
        Ok(())
    }
}
