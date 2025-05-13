use crate::interval_tree::{PyBatchHitSegments, PyHitSegments};
use biobit_collections_rs::interval_tree::{BatchHits, Hits};
use biobit_core_py::loc::{Interval, IntoPyInterval, PyInterval};
use biobit_core_py::pickle;
use biobit_core_py::utils::{ByPyPointer, type_hint_class_getitem};
use derive_getters::Dissolve;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyIterator, PyList, PyType};
use pyo3::{PyObject, pyclass, pymethods};

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

    fn append(&mut self, py: Python, interval: IntoPyInterval, data: PyObject) {
        self.intervals.push(interval.extract_py(py).rs);
        self.data.push(data);
    }

    fn extend(&mut self, items: PyObject) -> PyResult<()> {
        let before = self.intervals.len();
        Python::with_gil(|py| {
            for item_bound in items.bind(py).try_iter()? {
                let (interval, data) = item_bound?.extract::<(IntoPyInterval, PyObject)>()?;
                self.intervals.push(interval.extract_py(py).rs);
                self.data.push(data);
            }
            Ok(())
        })
        .inspect_err(|_| {
            self.intervals.truncate(before);
            self.data.truncate(before);
        })
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
            let data = slf.data.iter().map(ByPyPointer::from_ref);

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

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: PyObject) -> PyResult<PyObject> {
        type_hint_class_getitem(cls, args)
    }

    pub fn __repr__(&self) -> String {
        let len = self.intervals.len();
        format!("<Hits(len={})>", len)
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
#[derive(Dissolve)]
pub struct PyBatchHits {
    cache: Option<BatchHits<'static, i64, PyObject>>,
    intervals: Vec<PyInterval>,
    hits: Vec<PyObject>,
    index: Vec<usize>,
}

impl Default for PyBatchHits {
    fn default() -> Self {
        Self::new()
    }
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
        Self {
            cache: None,
            intervals: Vec::new(),
            hits: Vec::new(),
            index: vec![0],
        }
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

    pub fn append(&mut self, intervals: PyObject, data: PyObject) -> PyResult<()> {
        let before = self.intervals.len();
        Python::with_gil(|py| {
            let (mut intervals, mut data) = (intervals.bind(py).try_iter()?, data.bind(py).try_iter()?);
            for (interval, data) in intervals.by_ref().zip(data.by_ref()) {
                let interval = interval?.extract::<IntoPyInterval>()?.extract_py(py);
                let data = data?.extract::<PyObject>()?;

                self.intervals.push(interval);
                self.hits.push(data);
            }

            if intervals.next().is_some() || data.next().is_some() {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Mismatch between number of intervals and data objects for the query being appended.",
                ));
            }
            self.index.push(self.intervals.len());
            Ok(())
        }).inspect_err(|_| {
            self.intervals.truncate(before);
            self.hits.truncate(before);
        })
    }

    pub fn extend(&mut self, queries: Bound<PyAny>) -> PyResult<()> {
        let before = self.intervals.len();
        let index = self.index.len();
        for query in queries.try_iter()? {
            let (intervals, data) = query?.extract::<(PyObject, PyObject)>()?;
            self.append(intervals, data).inspect_err(|_| {
                self.intervals.truncate(before);
                self.hits.truncate(before);
                self.index.truncate(index);
            })?;
        }
        Ok(())
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
                    .map(ByPyPointer::from_ref)
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

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: PyObject) -> PyResult<PyObject> {
        type_hint_class_getitem(cls, args)
    }

    pub fn __repr__(&self) -> String {
        let len = self.__len__();
        let total_hits = self.intervals.len();
        format!("<BatchHits(queries={len}, total_hits={total_hits})>")
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
