use biobit_core_py::ngs::{MatesOrientation, PyLayout, Strandedness};
use biobit_core_py::parallelism;
use biobit_core_rs::LendingIterator;
use biobit_core_rs::source::{DynSource, Source};
use biobit_io_py::bam::IntoPyReader;
use biobit_io_py::fasta::PyIndexedSources;
use biobit_io_rs::bam::{strdeductor, transform};
use biobit_reat_rs::Reat;
use biobit_reat_rs::worker::{SourceArgs, SourceItem};
use eyre::{Result, eyre};
use higher_kinded_types::prelude::*;
use pyo3::prelude::*;
use rayon::ThreadPoolBuilder;

use crate::result::PySamplePileup;
use crate::selection::{IntoPySelector, PyMismatches};
use crate::task::PyTask;

type OrientedRecordSource = dyn Source<
        Args = SourceArgs<String>,
        Item = SourceItem,
        Iter = For!(<'borrow> = Box<dyn 'borrow + LendingIterator<Item = SourceItem>>),
    >;

#[pyclass(name = "Reat")]
pub struct PyReat {
    samples: Vec<Py<PyAny>>,
    reat: Reat<String, u64, u32, usize, Box<OrientedRecordSource>>,
}

impl PyReat {
    fn find_sample(&mut self, tag: &Py<PyAny>, py: Python) -> Result<Option<usize>> {
        for (ind, sample) in self.samples.iter().enumerate() {
            if sample.bind(py).eq(tag.bind(py))? {
                return Ok(Some(ind));
            }
        }
        Ok(None)
    }

    fn find_or_insert_sample(&mut self, tag: Py<PyAny>, py: Python) -> Result<usize> {
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
impl PyReat {
    #[new]
    #[pyo3(signature = (reference, selector = None, min_phred = 20, threads = -1))]
    pub fn new(
        reference: Py<PyIndexedSources>,
        selector: Option<IntoPySelector>,
        min_phred: u8,
        threads: isize,
        py: Python,
    ) -> Result<Self> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(parallelism::available(threads)?)
            .build()?;
        let reference = reference.borrow(py).rs.clone();
        let selector = selector
            .unwrap_or_else(|| PyMismatches::default().into())
            .rs;

        Ok(Self {
            samples: Vec::new(),
            reat: Reat::new(pool, reference, min_phred, selector),
        })
    }

    pub fn add_sources(
        mut slf: PyRefMut<Self>,
        tag: Py<PyAny>,
        sources: Vec<IntoPyReader>,
        layout: PyLayout,
    ) -> PyResult<PyRefMut<Self>> {
        let py = slf.py();
        let sample = slf.find_or_insert_sample(tag, py)?;
        let sources = sources
            .into_iter()
            .map(|source| to_oriented_records(py, source, layout))
            .collect::<Result<Vec<_>>>()?;

        slf.reat.register(sample, sources);
        Ok(slf)
    }

    pub fn run(mut slf: PyRefMut<Self>, tasks: Vec<PyTask>) -> PyResult<Vec<PySamplePileup>> {
        let tasks = tasks.into_iter().map(|task| task.rs).collect::<Vec<_>>();
        let py = slf.py();
        let results = slf.reat.run(tasks)?;
        let results = results
            .into_iter()
            .map(|selected| PySamplePileup::new(slf.samples[selected.tag].clone_ref(py), selected))
            .collect();
        Ok(results)
    }

    pub fn reset(mut slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf.samples.clear();
        slf.reat.reset();
        slf
    }
}

fn to_oriented_records(
    py: Python,
    source: IntoPyReader,
    layout: PyLayout,
) -> Result<Box<OrientedRecordSource>> {
    let source = source.0.borrow(py).clone().dissolve();

    match layout {
        PyLayout::Single { strandedness } => match strandedness.0 {
            Strandedness::Forward => Ok(source
                .with_transform(
                    transform::BundleByOrientation::new(strdeductor::deduce::se::forward),
                    (),
                )
                .to_dynsrc()
                .to_src()
                .boxed()),
            Strandedness::Reverse => Ok(source
                .with_transform(
                    transform::BundleByOrientation::new(strdeductor::deduce::se::reverse),
                    (),
                )
                .to_dynsrc()
                .to_src()
                .boxed()),
            Strandedness::Unstranded => Ok(source
                .with_transform(
                    transform::BundleByOrientation::new(strdeductor::deduce::se::unstranded),
                    (),
                )
                .to_dynsrc()
                .to_src()
                .boxed()),
        },
        PyLayout::Paired {
            strandedness,
            orientation,
        } => {
            if orientation.0 != MatesOrientation::Inward {
                return Err(eyre!(
                    "Only inward mates orientation is supported by REAT paired layouts"
                ));
            }
            match strandedness.0 {
                Strandedness::Forward => Ok(source
                    .with_transform(
                        transform::BundleByOrientation::new(strdeductor::deduce::pe::forward),
                        (),
                    )
                    .to_dynsrc()
                    .to_src()
                    .boxed()),
                Strandedness::Reverse => Ok(source
                    .with_transform(
                        transform::BundleByOrientation::new(strdeductor::deduce::pe::reverse),
                        (),
                    )
                    .to_dynsrc()
                    .to_src()
                    .boxed()),
                Strandedness::Unstranded => Ok(source
                    .with_transform(
                        transform::BundleByOrientation::new(strdeductor::deduce::pe::unstranded),
                        (),
                    )
                    .to_dynsrc()
                    .to_src()
                    .boxed()),
            }
        }
    }
}
