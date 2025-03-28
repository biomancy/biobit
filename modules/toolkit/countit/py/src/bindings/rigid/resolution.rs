use biobit_core_py::utils::{FallibleBound, ImportablePyModuleBuilder};
pub use biobit_countit_rs::rigid::resolution::{
    AnyOverlap, OverlapWeighted, Resolution, TopRanked,
};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[pyclass(name = "AnyOverlap")]
#[repr(transparent)]
#[derive(Default, Dissolve, From, Into)]
pub struct PyAnyOverlap(pub AnyOverlap);

#[pymethods]
impl PyAnyOverlap {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }
}

#[pyclass(name = "OverlapWeighted")]
#[repr(transparent)]
#[derive(Default, Dissolve, From, Into)]
pub struct PyOverlapWeighted(pub OverlapWeighted<usize>);

#[pymethods]
impl PyOverlapWeighted {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }
}

#[pyclass(name = "TopRanked")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyTopRanked(pub Box<dyn Resolution<usize, f64, PyObject>>);

#[pymethods]
impl PyTopRanked {
    #[new]
    pub fn new(priority: Vec<PyObject>) -> Self {
        let priority = Arc::new(priority);

        PyTopRanked(Box::new(TopRanked::new(
            move |mut ranks, elements: &[PyObject], partition: &[usize]| {
                ranks.clear();
                println!("TRYING TO ACQUIRE GIL!!");
                Python::with_gil(|py| {
                    let ranking: HashMap<_, usize> = priority
                        .iter()
                        .enumerate()
                        .map(|x| (FallibleBound(x.1.clone_ref(py).into_bound(py)), x.0))
                        .collect();
                    for ind in partition {
                        let element = FallibleBound(elements[*ind].clone_ref(py).into_bound(py));
                        match ranking.get(&element) {
                            Some(rank) => ranks.push(*rank),
                            None => {
                                log::warn!("Element not found in ranking: {:?}", element.0.str());
                                ranks.push(usize::MAX);
                            }
                        }
                    }
                });
                println!("GIL RELEASED!!");
                ranks
            },
        )))
    }
}

#[pyclass(name = "IntoResolution")]
#[repr(transparent)]
#[derive(From, Into)]
pub struct IntoPyResolution(pub Box<dyn Resolution<usize, f64, PyObject>>);

impl<'py> FromPyObject<'py> for IntoPyResolution {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let resolution = if obj.is_instance_of::<PyAnyOverlap>() {
            Box::new(obj.downcast::<PyAnyOverlap>()?.borrow().0.clone())
        } else if obj.is_instance_of::<PyOverlapWeighted>() {
            Box::new(obj.downcast::<PyOverlapWeighted>()?.borrow().0.clone())
        } else if obj.is_instance_of::<PyTopRanked>() {
            let obj = obj.downcast::<PyTopRanked>()?.borrow();
            dyn_clone::clone_box(&*obj.0)
        } else {
            return Err(PyValueError::new_err(format!(
                "Unknown resolution: {}",
                obj.get_type().name()?
            )));
        };
        Ok(IntoPyResolution(resolution))
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyAnyOverlap>()?
        .add_class::<PyOverlapWeighted>()?
        .add_class::<PyTopRanked>()?
        .add_class::<IntoPyResolution>()?
        .finish();

    Ok(module)
}
