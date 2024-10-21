use biobit_core_py::loc::{AsSegment, IntoPyOrientation, IntoPySegment, Segment};
use biobit_core_py::parallelism;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::WrapErr;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use rayon::ThreadPoolBuilder;

use crate::rigid::PyEngine;
pub use biobit_countit_rs::rigid::EngineBuilder;

#[pyclass(name = "EngineBuilder")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyEngineBuilder(pub EngineBuilder<String, usize, PyObject>);

#[pymethods]
impl PyEngineBuilder {
    #[new]
    pub fn new() -> Self {
        Self(EngineBuilder::default())
    }

    pub fn set_threads(mut slf: PyRefMut<Self>, threads: isize) -> PyResult<PyRefMut<Self>> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(parallelism::available(threads).unwrap())
            .build()
            .wrap_err_with(|| "Failed to build the thread pool")?;

        slf.0 = std::mem::take(&mut slf.0).set_thread_pool(pool);
        Ok(slf)
    }

    pub fn add_elements(
        mut slf: PyRefMut<Self>,
        elements: Vec<(
            PyObject,
            Vec<(String, IntoPyOrientation, Vec<IntoPySegment>)>,
        )>,
    ) -> PyRefMut<Self> {
        let py = slf.py();
        let elements = elements.into_iter().map(|(element, segments)| {
            let segments = segments
                .into_iter()
                .map(|(contig, orientation, segments)| {
                    let segments = segments
                        .into_iter()
                        .map(|segment| {
                            let segment = segment.0.borrow(py).rs;
                            Segment::new(segment.start() as usize, segment.end() as usize).unwrap()
                        })
                        .collect();
                    (contig, orientation.0 .0, segments)
                })
                .collect();
            (element, segments)
        });
        slf.0 = std::mem::take(&mut slf.0).add_elements(elements);
        slf
    }

    pub fn add_partitions(
        mut slf: PyRefMut<Self>,
        partitions: Vec<(String, IntoPySegment)>,
    ) -> PyRefMut<Self> {
        let py = slf.py();
        let partitions = partitions.into_iter().map(|(contig, segment)| {
            let segment = segment.0.borrow(py).rs;
            (
                contig,
                Segment::new(segment.start() as usize, segment.end() as usize).unwrap(),
            )
        });
        slf.0 = std::mem::take(&mut slf.0).add_partitions(partitions);
        slf
    }

    pub fn build(mut slf: PyRefMut<Self>) -> PyEngine {
        PyEngine(std::mem::take(&mut slf.0).build())
    }

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: PyObject, py: Python) -> PyResult<PyObject> {
        let locals = PyDict::new_bound(py);
        locals.set_item("cls", cls)?;
        locals.set_item("args", args)?;

        py.run_bound(
            r#"import types;result = types.GenericAlias(cls, args);"#,
            None,
            Some(&locals),
        )?;

        Ok(locals.get_item("result")?.unwrap().unbind())
    }
}
