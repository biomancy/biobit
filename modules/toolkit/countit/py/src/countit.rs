use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::Result;
use pyo3::prelude::*;
use rayon::ThreadPoolBuilder;

use biobit_core_py::{
    loc::{AsSegment, IntoPyOrientation, IntoPySegment, Segment},
    ngs::PyLayout,
    parallelism,
};
use biobit_core_py::loc::{IntoPyLocus, Locus};
use biobit_countit_rs::CountIt;
use biobit_io_py::bam::{IntoPyReader, utils::SegmentedAlignmentSource};

use super::result::PyCounts;

#[pyclass(name = "CountIt")]
#[repr(transparent)]
#[derive(Dissolve, From, Into)]
pub struct PyCountIt(
    CountIt<String, usize, f64, PyObject, PyObject, Box<SegmentedAlignmentSource>>,
);

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
        let source = biobit_io_py::bam::utils::to_alignment_segments(py, source, layout)?;

        slf.0.add_source(tag, source);
        Ok(slf)
    }

    pub fn run(&mut self, py: Python) -> PyResult<Vec<PyCounts>> {
        Ok(self.0.run()?.into_iter().map(|x| x.into_py(py)).collect())
    }
}
