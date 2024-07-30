use std::cmp::Ordering;

use eyre::{eyre, Result};
use pyo3::prelude::*;
use rayon::ThreadPoolBuilder;

use biobit_core_py::ngs::PyLayout;
use biobit_core_py::parallelism;
use biobit_io_py::bam::{IntoPyReader, utils::SegmentedAlignmentSource};
use biobit_ripper_rs::Ripper;

use super::config::PyConfig;

#[pyclass]
pub struct PyRipper {
    // PyObjects are not oredered/hashable, which is required for all Ripper sample tags
    // Workaround: store the tags in a Vec and search for them when needed
    samples: Vec<PyObject>,
    ripper: Ripper<String, usize, f64, usize, PyObject, Box<SegmentedAlignmentSource>>,
}

impl PyRipper {
    fn find_sample(&mut self, tag: &PyObject, py: Python) -> Result<Option<usize>> {
        for (ind, sample) in self.samples.iter().enumerate() {
            if sample.bind(py).compare(&tag)? == Ordering::Equal {
                return Ok(Some(ind));
            }
        }
        Ok(None)
    }

    fn find_or_insert_sample(&mut self, tag: PyObject, py: Python) -> Result<usize> {
        match self.find_sample(&tag, py)? {
            Some(ind) => Ok(ind),
            None => {
                self.samples.push(tag);
                Ok(self.samples.len() - 1)
            }
        }
    }
}

#[pymethods]
impl PyRipper {
    #[new]
    #[pyo3(signature = (threads = -1))]
    pub fn new(threads: isize) -> Result<Self> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(parallelism::available(threads)?)
            .build()?;
        Ok(PyRipper {
            samples: Vec::new(),
            ripper: Ripper::new(pool),
        })
    }

    pub fn add_partition<'a>(
        mut slf: PyRefMut<'a, Self>,
        contig: &'a str,
        start: usize,
        end: usize,
    ) -> PyRefMut<'a, Self> {
        slf.ripper.add_partition(contig.to_string(), start, end);
        slf
    }

    pub fn add_source(
        mut slf: PyRefMut<Self>,
        tag: PyObject,
        source: IntoPyReader,
        layout: PyLayout,
    ) -> PyResult<PyRefMut<Self>> {
        let py = slf.py();
        let sample = slf.find_or_insert_sample(tag, py)?;

        let source = biobit_io_py::bam::utils::to_alignment_segments(py, source, layout)?;

        slf.ripper.add_source(sample, source);
        Ok(slf)
    }

    pub fn add_sources(
        mut slf: PyRefMut<Self>,
        sample: PyObject,
        sources: Vec<IntoPyReader>,
        layout: PyLayout,
    ) -> PyResult<PyRefMut<Self>> {
        let py = slf.py();
        let sample = slf.find_or_insert_sample(sample, py)?;

        let sources = sources
            .into_iter()
            .map(|source| biobit_io_py::bam::utils::to_alignment_segments(py, source, layout))
            .collect::<Result<Vec<_>>>()?;

        slf.ripper.add_sources(sample, sources);
        Ok(slf)
    }

    pub fn add_comparison(
        mut slf: PyRefMut<Self>,
        tag: PyObject,
        signal: PyObject,
        control: PyObject,
        config: PyConfig,
    ) -> PyResult<PyRefMut<Self>> {
        let py = slf.py();

        let signal = slf.find_sample(&signal, py)?.ok_or_else(|| {
            eyre!("There are no registered samples equal to the requested signal tag.")
        })?;

        let control = slf.find_sample(&control, py)?.ok_or_else(|| {
            eyre!("There are no registered samples equal to the requested control tag.")
        })?;

        // Add the comparison normally
        slf.ripper
            .add_comparison(tag, &signal, &control, config.into())?;
        Ok(slf)
    }
}
