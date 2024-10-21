use crate::PyCounts;
use biobit_core_py::ngs::PyLayout;
use biobit_countit_rs::rigid::resolution;
pub use biobit_countit_rs::rigid::Engine;
use biobit_io_py::bam::IntoPyReader;
use derive_more::{From, Into};
use pyo3::prelude::*;

#[pyclass(name = "Engine")]
#[repr(transparent)]
#[derive(From, Into)]
pub struct PyEngine(pub Engine<String, usize, f64, PyObject>);

impl PyEngine {
    pub fn run(
        &mut self,
        sources: Vec<(PyObject, IntoPyReader, PyLayout)>,
        // resolution: Box<dyn Resolution<Idx, Cnts, Elt>>,
        py: Python,
    ) -> PyResult<Vec<PyCounts>> {
        let mut readers = Vec::with_capacity(sources.len());
        for (tag, source, layout) in sources {
            let source = biobit_io_py::bam::utils::to_alignment_segments(py, source, layout)?;
            readers.push((tag, source));
        }
        let resolution = Box::new(resolution::TopRanked::new(
            |ranks, elements| {
                let mut ranks = ranks;
                ranks.resize(elements.len(), 0);
                ranks
            },
            true,
        ));

        Ok(self
            .0
            .run(readers.into_iter(), resolution)?
            .into_iter()
            .map(|x| x.into_py(py))
            .collect())
    }
}
