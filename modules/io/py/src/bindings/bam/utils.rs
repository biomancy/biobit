use std::io;

use eyre::{eyre, Result};
use higher_kinded_types::prelude::*;
use pyo3::Python;

use biobit_core_py::ngs::{MatesOrientation, PyLayout, Strandedness};
use biobit_core_py::source::{DynSource, Source};
use biobit_core_py::LendingIterator;
use biobit_io_rs::bam::SegmentedAlignment;
use biobit_io_rs::bam::{strdeductor, transform};

use super::reader::IntoPyReader;

pub type SegmentedAlignmentBatch = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);
pub type SegmentedAlignmentSource = dyn Source<
    Args = For!(<'args> = (&'args String, usize, usize)),
    Item = SegmentedAlignmentBatch,
    Iter = For!(<'borrow> = Box<dyn 'borrow + LendingIterator<Item = SegmentedAlignmentBatch>>),
>;

pub fn to_alignment_segments(
    py: Python,
    source: IntoPyReader,
    layout: PyLayout,
) -> Result<Box<SegmentedAlignmentSource>> {
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
                Err(eyre!(
                    "Only inward mates orientation is supported by countit"
                ))?
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

    Ok(source)
}
