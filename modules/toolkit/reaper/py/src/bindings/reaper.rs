use std::cmp::Ordering;

use eyre::{Result, eyre};
use pyo3::prelude::*;
use rayon::ThreadPoolBuilder;

use biobit_core_py::ngs::PyLayout;
use biobit_core_py::parallelism;
use biobit_io_py::bam::{IntoPyReader, utils::SegmentedAlignmentSource};
use biobit_reaper_rs::Reaper;

use crate::PyHarvest;

use super::workload::PyWorkload;

#[pyclass(name = "Reaper")]
pub struct PyReaper {
    // PyObjects are not oredered/hashable, which is required for all Ripper sample tags
    // Workaround: store the tags in a Vec and search for them when needed
    samples: Vec<PyObject>,
    reaper: Reaper<String, usize, f32, usize, PyObject, Box<SegmentedAlignmentSource>>,
}

impl PyReaper {
    fn find_sample(&mut self, tag: &PyObject, py: Python) -> Result<Option<usize>> {
        for (ind, sample) in self.samples.iter().enumerate() {
            if sample.bind(py).compare(tag)? == Ordering::Equal {
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
impl PyReaper {
    #[new]
    #[pyo3(signature = (threads = -1))]
    pub fn new(threads: isize) -> Result<Self> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(parallelism::available(threads)?)
            .build()?;
        Ok(PyReaper {
            samples: Vec::new(),
            reaper: Reaper::new(pool),
        })
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

        slf.reaper.add_source(sample, source);
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

        slf.reaper.add_sources(sample, sources);
        Ok(slf)
    }

    pub fn add_comparison(
        mut slf: PyRefMut<Self>,
        tag: PyObject,
        signal: PyObject,
        control: PyObject,
        workload: PyWorkload,
    ) -> PyResult<PyRefMut<Self>> {
        let py = slf.py();

        let signal = slf.find_sample(&signal, py)?.ok_or_else(|| {
            eyre!("There are no registered samples equal to the requested signal tag.")
        })?;

        let control = slf.find_sample(&control, py)?.ok_or_else(|| {
            eyre!("There are no registered samples equal to the requested control tag.")
        })?;

        // Add the comparison normally
        slf.reaper
            .add_comparison(tag, &signal, &control, workload.into())?;
        Ok(slf)
    }

    pub fn run(mut slf: PyRefMut<Self>) -> PyResult<Vec<PyHarvest>> {
        let reaped = slf.reaper.run()?.into_iter().map(PyHarvest::from).collect();
        slf.reaper.reset();

        Ok(reaped)
    }

    pub fn reset(mut slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf.reaper.reset();
        slf
    }
}
