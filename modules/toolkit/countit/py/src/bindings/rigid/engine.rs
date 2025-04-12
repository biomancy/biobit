use crate::rigid::resolution::IntoPyResolution;
use crate::rigid::PyEngineBuilder;
use crate::PyCounts;
use biobit_core_py::ngs::PyLayout;
use biobit_core_py::utils::type_hint_class_getitem;
pub use biobit_countit_rs::rigid::Engine;
use biobit_io_py::bam::IntoPyReader;
use derive_more::{From, Into};
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass(name = "Engine")]
#[repr(transparent)]
#[derive(From, Into)]
pub struct PyEngine(pub Engine<String, usize, f64, PyObject>);

#[pymethods]
impl PyEngine {
    #[staticmethod]
    pub fn builder() -> PyEngineBuilder {
        PyEngineBuilder::new()
    }

    pub fn run(
        &mut self,
        sources: Vec<(PyObject, IntoPyReader, PyLayout)>,
        resolution: IntoPyResolution,
        py: Python,
    ) -> PyResult<Vec<PyCounts>> {
        let mut readers = Vec::with_capacity(sources.len());
        for (tag, source, layout) in sources {
            let source = biobit_io_py::bam::utils::to_alignment_segments(py, source, layout)?;
            readers.push((tag, source));
        }
        let result = py.allow_threads(|| self.0.run(readers.into_iter(), resolution.0))?;
        Ok(result.into_iter().map(PyCounts::from).collect())
    }

    #[classmethod]
    pub fn __class_getitem__(cls: Bound<PyType>, args: PyObject) -> PyResult<PyObject> {
        type_hint_class_getitem(cls, args)
    }
}
