pub use biobit_collections_rs::interval_tree::overlap::{Elements, Steps};
use biobit_core_py::fallible_py_runtime::FallibleBorrowed;
use biobit_core_py::loc::{IntoPySegment, PySegment};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyList, PySet};
use pyo3::{pyclass, pymethods, PyObject, PyTypeInfo};
use std::hash::{Hash, Hasher};

#[pyclass(name = "Elements")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyElements(pub Elements<i64, PyObject>);

impl PyElements {
    fn reset_with(
        &mut self,
        py: Python,
        segments: Vec<Vec<IntoPySegment>>,
        elements: Vec<Vec<PyObject>>,
    ) {
        self.0.reset();
        for (segments, elements) in segments.into_iter().zip(elements) {
            assert_eq!(segments.len(), elements.len());

            let mut adder = self.0.add();
            for (segment, element) in segments.into_iter().zip(elements) {
                adder.add(segment.0.into_bound(py).borrow().rs, element);
            }
            adder.finish();
        }
    }
}

#[pymethods]
impl PyElements {
    #[new]
    pub fn new() -> Self {
        Self(Elements::default())
    }

    #[staticmethod]
    pub fn from_existent(
        py: Python,
        segments: Vec<Vec<IntoPySegment>>,
        elements: Vec<Vec<PyObject>>,
    ) -> Self {
        let mut result = PyElements::new();
        result.reset_with(py, segments, elements);
        result
    }

    #[getter]
    pub fn segments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let result = PyList::empty_bound(py);

        for x in self.0.segments() {
            let inner = PyList::new_bound(py, x.iter().map(|y| PySegment::from(*y).into_py(py)));
            result.append(inner)?;
        }
        Ok(result)
    }

    #[getter]
    pub fn elements<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let result = PyList::empty_bound(py);

        for x in self.0.annotations() {
            let inner = PyList::new_bound(py, x.iter().map(|y| y.clone_ref(py)));
            result.append(inner)?;
        }
        Ok(result)
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::empty_bound(py);
        for x in self.0.iter() {
            let segments =
                PyList::new_bound(py, x.0.iter().map(|y| PySegment::from(*y).into_py(py)));
            let annotations = PyList::new_bound(py, x.1.iter().map(|y| y.clone_ref(py)));
            result.append((segments, annotations))?
        }

        PyIterator::from_bound_object(result.as_any())
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

    pub fn __getstate__(&self, py: Python) -> PyResult<(PyObject, PyObject)> {
        Ok((
            self.segments(py)?.to_object(py),
            self.elements(py)?.to_object(py),
        ))
    }

    pub fn __setstate__(
        &mut self,
        py: Python,
        state: (Vec<Vec<IntoPySegment>>, Vec<Vec<PyObject>>),
    ) {
        self.reset_with(py, state.0, state.1);
    }
}

#[pyclass(name = "Steps")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PySteps(Vec<Vec<(PySegment, Vec<PyObject>)>>);

#[pymethods]
impl PySteps {
    #[new]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn build<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        elements: &PyElements,
        query: Vec<IntoPySegment>,
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
                let segment = PySegment::new(x.0, x.1)?;
                let objects =
                    x.2.iter()
                        .map(|y: &FallibleBorrowed| (*y.0).clone().unbind().to_object(py))
                        .collect();
                hits.push((segment, objects));
            }
            slf.0.push(hits);
        }

        Ok(slf)
    }

    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let result = PyList::empty_bound(py);
        for sample in self.0.iter() {
            let mut inner = PyList::empty_bound(py);
            for (segment, objects) in sample.iter() {
                let pyset = PySet::new_bound(py, objects.into_iter())?;
                inner.append((segment.into_py(py), pyset))?;
            }
            result.append(inner)?;
        }
        PyIterator::from_bound_object(result.as_any())
    }
}

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.overlap", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyElements>()?;
    module.add_class::<PySteps>()?;

    for typbj in [
        PyElements::type_object_bound(parent.py()),
        PySteps::type_object_bound(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
