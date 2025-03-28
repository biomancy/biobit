pub use biobit_collections_rs::interval_tree::overlap::{Elements, Steps};
use biobit_core_py::loc::{IntoPyInterval, PyInterval};
use biobit_core_py::utils::{FallibleBorrowed, ImportablePyModuleBuilder};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyList, PySet};
use pyo3::{pyclass, pymethods, PyObject};

#[pyclass(name = "Elements")]
#[repr(transparent)]
#[derive(Default, Dissolve, From, Into)]
pub struct PyElements(pub Elements<i64, PyObject>);

impl PyElements {
    fn reset_with(
        &mut self,
        py: Python,
        intervals: Vec<Vec<IntoPyInterval>>,
        elements: Vec<Vec<PyObject>>,
    ) {
        self.0.clear();
        for (intervals, elements) in intervals.into_iter().zip(elements) {
            assert_eq!(intervals.len(), elements.len());

            let mut adder = self.0.add();
            for (interval, element) in intervals.into_iter().zip(elements) {
                adder.add(interval.0.into_bound(py).borrow().rs, element);
            }
            adder.finish();
        }
    }
}

#[pymethods]
impl PyElements {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[staticmethod]
    pub fn from_existent(
        py: Python,
        intervals: Vec<Vec<IntoPyInterval>>,
        elements: Vec<Vec<PyObject>>,
    ) -> Self {
        let mut result = PyElements::new();
        result.reset_with(py, intervals, elements);
        result
    }

    #[getter]
    pub fn intervals<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let result = PyList::empty(py);

        for x in self.0.intervals() {
            let inner = PyList::new(py, x.iter().map(|y| PyInterval::from(*y)));
            result.append(inner?)?;
        }
        Ok(result)
    }

    #[getter]
    pub fn elements<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let result = PyList::empty(py);

        for x in self.0.annotations() {
            let inner = PyList::new(py, x.iter().map(|y| y.clone_ref(py)));
            result.append(inner?)?;
        }
        Ok(result)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::empty(py);
        for x in self.0.iter() {
            let intervals = PyList::new(py, x.0.iter().map(|y| PyInterval::from(*y)));
            let annotations = PyList::new(py, x.1.iter().map(|y| y.clone_ref(py)));
            result.append((intervals?, annotations?))?
        }

        PyIterator::from_object(result.as_any())
    }

    pub fn __eq__(&self, py: Python, other: &PyElements) -> PyResult<bool> {
        if self.__len__() != other.__len__() {
            return Ok(false);
        }

        for (x, y) in self.0.iter().zip(other.0.iter()) {
            if x.0 != y.0 {
                return Ok(false);
            }
            for (xobj, yobj) in x.1.iter().zip(y.1.iter()) {
                let (xobj, yobj) = (xobj.bind(py), yobj.bind(py));
                if !xobj.eq(yobj)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<(Py<PyList>, Py<PyList>)> {
        Ok((self.intervals(py)?.unbind(), self.elements(py)?.unbind()))
    }

    pub fn __setstate__(
        &mut self,
        py: Python,
        state: (Vec<Vec<IntoPyInterval>>, Vec<Vec<PyObject>>),
    ) {
        self.reset_with(py, state.0, state.1);
    }
}

#[pyclass(name = "Steps")]
#[repr(transparent)]
#[derive(Default, Dissolve, From, Into)]
pub struct PySteps(Vec<Vec<(PyInterval, Vec<PyObject>)>>);

#[pymethods]
impl PySteps {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        elements: &PyElements,
        query: Vec<IntoPyInterval>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        let mut steps = Steps::default();

        let dummies = elements
            .0
            .iter()
            .map(|x| {
                (
                    x.0,
                    x.1.iter()
                        .map(|y| FallibleBorrowed(y.bind_borrowed(py)))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        if query.len() != dummies.len() {
            return Err(PyValueError::new_err(
                "Query length does not match elements",
            ));
        }

        let query = query
            .iter()
            .map(|x| x.0.bind(py).borrow().rs)
            .collect::<Vec<_>>();

        steps.build(
            query
                .iter()
                .zip(dummies.iter().map(|(x, y)| (*x, y.as_slice()))),
        );

        slf.0.clear();
        for iter in steps.iter() {
            let mut hits = Vec::new();
            for x in iter {
                let interval = PyInterval::new(x.0, x.1)?;
                let objects =
                    x.2.iter()
                        .map(|y: &FallibleBorrowed| (*y.0).clone().unbind())
                        .collect();
                hits.push((interval, objects));
            }
            slf.0.push(hits);
        }

        Ok(slf)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::empty(py);
        for sample in self.0.iter() {
            let inner = PyList::empty(py);
            for (interval, objects) in sample.iter() {
                let pyset = PySet::new(py, objects)?;
                inner.append((*interval, pyset))?;
            }
            result.append(inner)?;
        }
        PyIterator::from_object(result.as_any())
    }

    pub fn __eq__(&self, py: Python, other: &PySteps) -> PyResult<bool> {
        if self.0.len() != other.0.len() {
            return Ok(false);
        };

        for (x, y) in self.0.iter().zip(other.0.iter()) {
            if x.len() != y.len() {
                return Ok(false);
            }
            for (x, y) in x.iter().zip(y.iter()) {
                if x.0 != y.0 {
                    return Ok(false);
                }
                if x.1.len() != y.1.len() {
                    return Ok(false);
                }
                for (x, y) in x.1.iter().zip(y.1.iter()) {
                    if !x.bind(py).eq(y.bind(py))? {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    pub fn __getstate__(&self) -> Vec<Vec<(PyInterval, Vec<PyObject>)>> {
        self.0.clone()
    }

    pub fn __setstate__(&mut self, state: Vec<Vec<(PyInterval, Vec<PyObject>)>>) {
        self.0 = state;
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyElements>()?
        .add_class::<PySteps>()?
        .finish();

    Ok(module)
}
