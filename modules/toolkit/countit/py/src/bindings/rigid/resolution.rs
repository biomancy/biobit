use biobit_core_py::fallible_py_runtime::FallibleBound;
pub use biobit_countit_rs::rigid::resolution::{
    AnyOverlap, OverlapWeighted, Resolution, TopRanked,
};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass(name = "AnyOverlap")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyAnyOverlap(pub AnyOverlap);

#[pymethods]
impl PyAnyOverlap {
    #[new]
    pub fn new(downscale_multimappers: bool) -> Self {
        AnyOverlap::new(downscale_multimappers).into()
    }
}

#[pyclass(name = "OverlapWeighted")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyOverlapWeighted(pub OverlapWeighted<usize>);

#[pymethods]
impl PyOverlapWeighted {
    #[new]
    pub fn new(downscale_multimappers: bool) -> Self {
        OverlapWeighted::new(downscale_multimappers).into()
    }
}

#[pyclass(name = "TopRanked")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyTopRanked(pub Box<dyn Resolution<usize, f64, PyObject>>);

#[pymethods]
impl PyTopRanked {
    #[new]
    pub fn new(priority: Vec<PyObject>, downscale_multimappers: bool) -> Self {
        PyTopRanked(Box::new(TopRanked::new(
            move |mut ranks, elements: &[PyObject]| {
                ranks.clear();
                Python::with_gil(|py| {
                    let ranking: HashMap<_, usize> = priority
                        .iter()
                        .enumerate()
                        .map(|x| (FallibleBound(x.1.clone_ref(py).into_bound(py)), x.0))
                        .collect();
                    for element in elements {
                        let element = FallibleBound(element.clone_ref(py).into_bound(py));
                        match ranking.get(&element) {
                            Some(rank) => ranks.push(*rank),
                            None => {
                                log::warn!("Element not found in ranking: {:?}", element.0.str());
                                ranks.push(usize::MAX);
                            }
                        }
                    }
                });
                ranks
            },
            downscale_multimappers,
        )))
    }
}

// // MUST be proper structures (even if they are super simple)
// // An IntoResolution struct that can be constructed from different things (e.g., structures, strings, etc.)
//
//
//
//
//
// pub fn register<'b>(
//     parent: &Bound<'b, PyModule>,
//     sysmod: &Bound<PyAny>,
// ) -> PyResult<Bound<'b, PyModule>> {
//     let name = format!("{}.resolution", parent.name()?);
//     let module = PyModule::new_bound(parent.py(), &name)?;
//
//     // module.add_class::<PyEngine>()?;
//     // module.add_class::<PyEngineBuilder>()?;
//
//     parent.add_submodule(&module)?;
//     sysmod.set_item(module.name()?, &module)?;
//
//     Ok(module)
// }
