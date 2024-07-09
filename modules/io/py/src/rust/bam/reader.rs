use std::io;
use std::path::PathBuf;

use pyo3::prelude::*;

use biobit_io_rs::bam::Reader as RsReader;
use biobit_io_rs::bam::ReaderBuilder as RsReaderBuilder;

#[pyclass(eq, ord, frozen, hash)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Reader {
    filename: PathBuf,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
    batch_size: usize,
}

#[pymethods]
impl Reader {
    #[new]
    pub fn new(
        filename: PathBuf,
        inflags: u16,
        exflags: u16,
        minmapq: u8,
        batch_size: usize,
    ) -> Self {
        Self {
            filename,
            inflags,
            exflags,
            minmapq,
            batch_size,
        }
    }
}

impl TryFrom<Reader> for RsReader {
    type Error = io::Error;

    fn try_from(value: Reader) -> Result<Self, Self::Error> {
        RsReaderBuilder::new(value.filename)
            .with_inflags(value.inflags)
            .with_exflags(value.exflags)
            .with_minmapq(value.minmapq)
            .with_batch_size(value.batch_size)
            .build()
    }
}
