use std::collections::HashMap;

use derive_getters::Dissolve;
use derive_more::{From, Into};
use noodles::bam::Record;
use pyo3::prelude::*;
use rayon::{max_num_threads, ThreadPoolBuilder};

use biobit_collections_rs::genomic_index::GenomicIndex;
use biobit_collections_rs::interval_tree::{Builder, LapperBuilder, LapperTree};
use biobit_core_py::seqlib::{strandedness, MatesOrientation, Strandedness};
use biobit_core_py::{
    loc::{Locus, Orientation, RsLocus, RsSegment, Segment},
    seqlib::SeqLib,
};
use biobit_countit_rs::CountIt as RsCountIt;
use biobit_io_py::bam::{AdaptersForIndexedBAM, Reader, RsReader};

#[pyclass]
#[derive(Dissolve, From, Into)]
pub struct CountIt(RsCountIt<PyObject, f64, String, LapperTree<usize, usize>>);

#[pymethods]
impl CountIt {
    #[new]
    pub fn new(
        data: HashMap<(String, Orientation), Vec<(PyObject, Vec<Segment>)>>,
        threads: i16,
    ) -> PyResult<Self> {
        let mut gindex = GenomicIndex::new();

        let mut all_data = Vec::new();
        for ((contig, orientation), data) in data {
            let mut tree = LapperBuilder::new();
            for d in data {
                let ind = all_data.len();
                all_data.push(d.0);
                for segment in d.1 {
                    let segment =
                        RsSegment::new(segment.start() as usize, segment.end() as usize).unwrap();
                    tree = tree.add(&segment, ind);
                }
            }
            gindex.set(contig, orientation.into(), tree.build());
        }

        let threads = if threads == 0 {
            1
        } else if threads < 0 {
            let max = max_num_threads() as isize;
            (max - threads as isize + 1).max(1) as usize
        } else {
            threads as usize
        };

        Ok(RsCountIt::new(
            ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("Failed to create thread pool"),
            all_data,
            gindex,
        )
        .into())
    }

    pub fn count(
        &mut self,
        py: Python<'_>,
        sources: Vec<(String, Reader, SeqLib)>,
        partitions: Vec<Locus>,
    ) -> PyResult<()> {
        let partitions: Vec<_> = partitions
            .into_iter()
            .map(|x| {
                let segment = x.segment.borrow(py);
                RsLocus {
                    contig: x.contig,
                    segment: RsSegment::new(segment.start() as usize, segment.end() as usize)
                        .unwrap(),
                    orientation: x.orientation.into(),
                }
            })
            .collect();

        let mut ids = Vec::with_capacity(sources.len());
        let mut box_sources = Vec::with_capacity(sources.len());

        for (id, bam, seqlib) in sources {
            ids.push(id);

            let bam: RsReader = bam.try_into()?;
            // let seqlib = Into::<RsSeqLib>::into(seqlib);

            let source = match seqlib {
                SeqLib::Single { strandedness } => match strandedness {
                    Strandedness::Forward => bam
                        .se_alignment_segments(|x: &Record| {
                            strandedness::deduce::se::forward(x.flags().is_reverse_complemented())
                        })
                        .boxed(),
                    Strandedness::Reverse => bam
                        .se_alignment_segments(|x: &Record| {
                            strandedness::deduce::se::reverse(x.flags().is_reverse_complemented())
                        })
                        .boxed(),
                    Strandedness::Unstranded => {
                        unimplemented!("Unstranded libraries are not supported")
                    }
                },
                SeqLib::Paired {
                    strandedness,
                    orientation,
                } => {
                    if orientation != MatesOrientation::Inward {
                        unimplemented!("Unsupported mates orientation")
                    }
                    let bam = bam.pe_bundled();
                    match strandedness {
                        Strandedness::Forward => bam
                            .pe_alignment_segments(|x: &Record| {
                                strandedness::deduce::pe::forward(
                                    x.flags().is_first_segment(),
                                    x.flags().is_reverse_complemented(),
                                )
                            })
                            .boxed(),
                        Strandedness::Reverse => bam
                            .pe_alignment_segments(|x: &Record| {
                                strandedness::deduce::pe::reverse(
                                    x.flags().is_first_segment(),
                                    x.flags().is_reverse_complemented(),
                                )
                            })
                            .boxed(),
                        Strandedness::Unstranded => {
                            unimplemented!("Unstranded libraries are not supported")
                        }
                    }
                }
            };

            box_sources.push(source);
        }

        self.0.count(box_sources, ids, &partitions);
        Ok(())
    }
}
