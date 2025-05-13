use biobit_collections_rs::interval_tree::{BatchHitSegments, HitSegments};
use biobit_core_py::loc::PyInterval;
use biobit_core_py::utils::type_hint_class_getitem;
use derive_getters::Dissolve;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenSet, PyIterator, PyList, PyType};
use pyo3::{Py, PyObject, PyResult, Python, pyclass, pymethods};
use std::hash::Hash;
use std::ops::Deref;

#[pyclass(name = "HitSegments")]
#[derive(Default, Dissolve)]
pub struct PyHitSegments {
    cache: Option<HitSegments<'static, i64, usize>>,
    segments: Vec<PyInterval>,
    data: Vec<Py<PyFrozenSet>>,
}

impl PyHitSegments {
    pub fn take<'tree, T: Hash + Eq>(&mut self) -> HitSegments<'tree, i64, T> {
        self.cache.take().unwrap_or_default().recycle()
    }

    pub fn reset<T: Hash + Eq + Deref<Target = PyObject>>(
        &mut self,
        py: Python<'_>,
        segments: HitSegments<'_, i64, T>,
    ) -> PyResult<()> {
        self.segments.clear();
        self.data.clear();
        self.segments
            .extend(segments.segments().iter().map(|x| PyInterval::from(*x)));

        for data_set in segments.data().iter() {
            self.data
                .push(PyFrozenSet::new(py, data_set.iter().map(|x| x.clone_ref(py)))?.unbind());
        }

        self.cache = Some(segments.recycle());
        Ok(())
    }
}

#[pymethods]
impl PyHitSegments {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn segments<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, self.segments.iter().cloned())
    }

    pub fn data<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, self.data.iter())
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.data.clear();
    }

    pub fn __len__(&self) -> usize {
        self.segments.len()
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::new(py, self.segments.iter().cloned().zip(self.data.iter()))?;
        result.try_iter()
    }

    pub fn __eq__(&self, other: &PyHitSegments) -> PyResult<bool> {
        if self.segments != other.segments || self.data.len() != other.data.len() {
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

    pub fn __getstate__<'a>(
        &self,
        py: Python<'a>,
    ) -> PyResult<(Bound<'a, PyList>, Bound<'a, PyList>)> {
        Ok((self.segments(py)?, self.data(py)?))
    }

    pub fn __setstate__(&mut self, state: (Vec<PyInterval>, Vec<Py<PyFrozenSet>>)) -> PyResult<()> {
        self.segments = state.0;
        self.data = state.1;
        Ok(())
    }
}

#[pyclass(name = "BatchHitSegments")]
#[derive(Dissolve)]
pub struct PyBatchHitSegments {
    cache: Option<BatchHitSegments<'static, i64, usize>>,
    segments: Vec<PyInterval>,
    data: Vec<Py<PyFrozenSet>>,
    index: Vec<usize>,
}

impl Default for PyBatchHitSegments {
    fn default() -> Self {
        Self::new()
    }
}

impl PyBatchHitSegments {
    pub fn take<'tree, T: Hash + Eq>(&mut self) -> BatchHitSegments<'tree, i64, T> {
        self.cache.take().unwrap_or_default().recycle()
    }

    pub fn reset<T: Hash + Eq + Deref<Target = PyObject>>(
        &mut self,
        py: Python<'_>,
        segments: BatchHitSegments<'_, i64, T>,
    ) -> PyResult<()> {
        self.clear();

        for (bsegment, bdata) in segments.iter() {
            self.segments
                .extend(bsegment.iter().cloned().map(PyInterval::from));

            for segment_data in bdata.iter() {
                self.data.push(
                    PyFrozenSet::new(py, segment_data.iter().map(|x| x.clone_ref(py)))?.unbind(),
                );
            }
            self.index.push(self.segments.len());
        }

        self.cache = Some(segments.recycle());
        Ok(())
    }
}

#[pymethods]
impl PyBatchHitSegments {
    #[new]
    pub fn new() -> Self {
        Self {
            cache: None,
            segments: Vec::new(),
            data: Vec::new(),
            index: vec![0],
        }
    }

    pub fn segments<'a>(&self, py: Python<'a>, i: usize) -> PyResult<Bound<'a, PyList>> {
        if i + 1 >= self.index.len() {
            Err(pyo3::exceptions::PyIndexError::new_err(
                "Index out of bounds",
            ))
        } else {
            PyList::new(
                py,
                self.segments[self.index[i]..self.index[i + 1]]
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
            PyList::new(py, self.data[self.index[i]..self.index[i + 1]].iter())
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.data.clear();
        self.index.clear();
        self.index.push(0);
    }

    pub fn __len__(&self) -> usize {
        self.index.len() - 1
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::empty(py);
        for i in 0..self.index.len() - 1 {
            result.append((self.segments(py, i)?, self.data(py, i)?))?;
        }
        result.try_iter()
    }

    pub fn __eq__(&self, other: &PyBatchHitSegments) -> PyResult<bool> {
        if self.index != other.index
            || self.segments != other.segments
            || self.data.len() != other.data.len()
        {
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

    pub fn __getstate__<'a>(
        &self,
        py: Python<'a>,
    ) -> PyResult<(Bound<'a, PyList>, Bound<'a, PyList>, Bound<'a, PyList>)> {
        Ok((
            PyList::new(py, self.segments.iter().cloned())?,
            PyList::new(py, self.data.iter())?,
            PyList::new(py, self.index.iter())?,
        ))
    }

    pub fn __setstate__(
        &mut self,
        state: (Vec<PyInterval>, Vec<Py<PyFrozenSet>>, Vec<usize>),
    ) -> PyResult<()> {
        self.segments = state.0;
        self.data = state.1;
        self.index = state.2;
        Ok(())
    }
}
