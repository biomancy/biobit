use crate::interval_tree::{PyBatchHits, PyHits};
use biobit_collections_rs::interval_tree::{Bits, BitsBuilder, Builder, ITree};
use biobit_core_py::loc::{IntoPyInterval, PyInterval};
use biobit_core_py::utils::type_hint_class_getitem;
use derive_getters::Dissolve;
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyType};

#[pyclass(name = "BitsBuilder")]
#[derive(Default, Dissolve)]
pub struct PyBitsBuilder {
    builder: BitsBuilder<i64, PyObject>,
}

#[pymethods]
impl PyBitsBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    fn add<'a>(
        mut slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        interval: IntoPyInterval,
        data: PyObject,
    ) -> PyRefMut<'a, Self> {
        let interval = interval.extract_py(py).rs;
        slf.builder = std::mem::take(&mut slf.builder).add(interval, data);
        slf
    }

    fn extend<'a>(
        mut slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        data: Bound<'a, PyAny>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        for item in data.try_iter()? {
            let (interval, data) = item?.extract::<(IntoPyInterval, PyObject)>()?;
            let interval = interval.extract_py(py).rs;
            slf.builder = std::mem::take(&mut slf.builder).add(interval, data);
        }
        Ok(slf)
    }

    fn build(mut slf: PyRefMut<Self>) -> PyBits {
        let tree = std::mem::take(&mut slf.builder).build();
        PyBits { tree }
    }

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: PyObject) -> PyResult<PyObject> {
        type_hint_class_getitem(cls, args)
    }
}

#[pyclass(name = "Bits")]
#[derive(Dissolve)]
pub struct PyBits {
    tree: Bits<i64, PyObject>,
}

#[pymethods]
impl PyBits {
    #[staticmethod]
    pub fn builder() -> PyBitsBuilder {
        PyBitsBuilder::new()
    }

    pub fn data<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, self.tree.data().iter())
    }

    pub fn intervals<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        let intervals = self.tree.intervals().map(PyInterval::from);
        let pylist = PyList::empty(py);
        for interval in intervals {
            pylist.append(interval)?;
        }
        Ok(pylist)
    }

    pub fn records<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        let pylist = PyList::empty(py);
        for record in self.tree.records() {
            pylist.append((PyInterval::from(record.0), record.1.clone_ref(py)))?;
        }
        Ok(pylist)
    }

    #[pyo3(signature = (interval, into = None))]
    pub fn intersect_interval<'py>(
        &'py self,
        py: Python<'py>,
        interval: IntoPyInterval,
        into: Option<Py<PyHits>>,
    ) -> PyResult<Py<PyHits>> {
        let into = match into {
            Some(into) => into,
            None => Py::new(py, PyHits::default())?,
        };
        let interval = interval.extract_py(py).rs;

        {
            let mut binding = into.borrow_mut(py);
            let mut buffer = binding.take();
            self.tree.intersect_interval(&interval, &mut buffer);
            binding.reset(py, buffer);
        }

        Ok(into)
    }

    #[pyo3(signature = (intervals, into = None))]
    pub fn batch_intersect_intervals<'py>(
        &'py self,
        py: Python<'py>,
        intervals: Vec<IntoPyInterval>,
        into: Option<Py<PyBatchHits>>,
    ) -> PyResult<Py<PyBatchHits>> {
        let into = match into {
            Some(into) => into,
            None => Py::new(py, PyBatchHits::default())?,
        };
        let intervals: Vec<_> = intervals
            .into_iter()
            .map(|interval| interval.extract_py(py).rs)
            .collect();

        {
            let mut binding = into.borrow_mut(py);
            let mut buffer = binding.take();
            self.tree.batch_intersect_intervals(&intervals, &mut buffer);
            binding.reset(py, buffer);
        }

        Ok(into)
    }

    pub fn __eq__(&self, other: &PyBits) -> PyResult<bool> {
        if self.tree.len() != other.tree.len() {
            return Ok(false);
        }

        if !self
            .tree
            .intervals()
            .zip(other.tree.intervals())
            .all(|(x, y)| x == y)
        {
            return Ok(false);
        }

        Python::with_gil(|py| -> PyResult<bool> {
            for (l, r) in self.tree.data().iter().zip(other.tree.data().iter()) {
                if !l.bind(py).eq(r.bind(py))? {
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

    #[staticmethod]
    pub fn _from_pickle(state: Bound<PyAny>) -> PyResult<Self> {
        let mut builder = BitsBuilder::default();
        for item in state.try_iter()? {
            let (interval, data) = item?.extract::<(IntoPyInterval, PyObject)>()?;
            let interval = interval.extract_py(state.py()).rs;
            builder = builder.add(interval, data);
        }
        let tree = builder.build();
        Ok(Self { tree })
    }

    pub fn __reduce__(&self, py: Python) -> PyResult<(PyObject, (Py<PyList>,))> {
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (self.records(py)?.unbind(),),
        ))
    }
}
