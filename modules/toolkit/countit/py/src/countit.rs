use std::io;

use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::{eyre, Result};
use pyo3::prelude::*;
use rayon::ThreadPoolBuilder;

use biobit_core_py::{
    LendingIterator,
    loc::{AsSegment, IntoPyOrientation, IntoPySegment, Segment},
    ngs::{MatesOrientation, PyLayout, Strandedness},
    parallelism,
    source::{DynSource, Source},
};
use biobit_core_py::loc::{IntoPyLocus, Locus};
use biobit_countit_rs::CountIt;
use biobit_io_py::bam::{SegmentedAlignment, IntoPyReader, strdeductor, transform};

use super::result::PyCounts;

type PySourceItem = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);
type PySource = Box<
    dyn Source<
        Args = For!(<'args> = (&'args String, usize, usize)),
        Item = PySourceItem,
        Iter = For!(<'borrow> = Box<dyn 'borrow + LendingIterator<Item = PySourceItem>>),
    >,
>;

#[pyclass(name = "CountIt")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyCountIt(CountIt<String, usize, f64, PyObject, PyObject, PySource>);

#[pymethods]
impl PyCountIt {
    #[new]
    #[pyo3(signature = (threads = -1))]
    pub fn new(threads: isize) -> Result<Self> {
        Ok(CountIt::new(
            ThreadPoolBuilder::new()
                .num_threads(parallelism::available(threads)?)
                .build()?,
        )
        .into())
    }
    pub fn add_annotation(
        mut slf: PyRefMut<Self>,
        data: PyObject,
        intervals: Vec<(String, IntoPyOrientation, Vec<IntoPySegment>)>,
    ) -> PyRefMut<Self> {
        let py = slf.py();

        let intervals = intervals
            .into_iter()
            .map(|(contig, orientation, segments)| {
                let segments = segments.into_iter().map(|segment| {
                    let segment = segment.0.borrow(py).rs;
                    Segment::new(segment.start() as usize, segment.end() as usize).unwrap()
                });
                (contig, orientation.0 .0, segments)
            });
        slf.0.add_annotation(data, intervals);

        slf
    }

    pub fn add_partition(mut slf: PyRefMut<Self>, partition: IntoPyLocus) -> PyRefMut<Self> {
        let partition = partition.0.borrow(slf.py());
        let segment = partition.segment.borrow(slf.py()).rs;

        let partition = Locus::new(
            partition.contig.clone(),
            Segment::new(segment.start() as usize, segment.end() as usize).unwrap(),
            partition.orientation.0,
        );

        slf.0.add_partition(partition);
        slf
    }

    pub fn add_source(
        mut slf: PyRefMut<Self>,
        tag: PyObject,
        source: IntoPyReader,
        layout: PyLayout,
    ) -> PyResult<PyRefMut<Self>> {
        let py = slf.py();
        let source = source.0.borrow(py).clone().dissolve();

        let source = match layout {
            PyLayout::Single { strandedness } => match strandedness.0 {
                Strandedness::Forward => source
                    .with_transform(
                        transform::ExtractAlignmentSegments::new(strdeductor::deduce::se::forward),
                        (),
                    )
                    .to_dynsrc()
                    .to_src()
                    .boxed(),
                Strandedness::Reverse => source
                    .with_transform(
                        transform::ExtractAlignmentSegments::new(strdeductor::deduce::se::reverse),
                        (),
                    )
                    .to_dynsrc()
                    .to_src()
                    .boxed(),
                Strandedness::Unstranded => {
                    return Err(eyre!("Unstranded libraries are not supported by countit"))?;
                }
            },
            PyLayout::Paired {
                strandedness,
                orientation,
            } => {
                if orientation.0 != MatesOrientation::Inward {
                    return Err(eyre!(
                        "Only inward mates orientation is supported by countit"
                    ))?;
                }

                let source = source.with_transform(transform::BundleMates::default(), ());

                match strandedness.0 {
                    Strandedness::Forward => source
                        .with_transform(
                            transform::ExtractPairedAlignmentSegments::new(
                                strdeductor::deduce::pe::forward,
                            ),
                            (),
                        )
                        .to_dynsrc()
                        .to_src()
                        .boxed(),
                    Strandedness::Reverse => source
                        .with_transform(
                            transform::ExtractPairedAlignmentSegments::new(
                                strdeductor::deduce::pe::reverse,
                            ),
                            (),
                        )
                        .to_dynsrc()
                        .to_src()
                        .boxed(),
                    Strandedness::Unstranded => {
                        return Err(eyre!("Unstranded libraries are not supported by countit"))?;
                    }
                }
            }
        };

        slf.0.add_source(tag, source);
        Ok(slf)
    }

    pub fn run(&mut self, py: Python) -> PyResult<Vec<PyCounts>> {
        Ok(self.0.run()?.into_iter().map(|x| x.into_py(py)).collect())
    }
}
