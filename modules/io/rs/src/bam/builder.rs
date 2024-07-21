use std::io;
use std::path::PathBuf;

use crate::bam::Reader;

use super::indexed_reader::IndexedReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderBuilder {
    filename: PathBuf,
    inflags: Option<u16>,
    exflags: Option<u16>,
    minmapq: Option<u8>,
    batch_size: Option<usize>,
}

impl ReaderBuilder {
    const DEFAULT_BATCH_SIZE: usize = 1024;

    pub fn new<T: Into<PathBuf>>(filename: T) -> Self {
        Self {
            filename: filename.into(),
            inflags: None,
            exflags: None,
            minmapq: None,
            batch_size: None,
        }
    }

    pub fn with_inflags(mut self, inflags: u16) -> Self {
        self.inflags = Some(inflags);
        self
    }

    pub fn with_exflags(mut self, exflags: u16) -> Self {
        self.exflags = Some(exflags);
        self
    }

    pub fn with_minmapq(mut self, minmapq: u8) -> Self {
        self.minmapq = Some(minmapq);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn build(self) -> io::Result<Reader> {
        let mut reader = IndexedReader::new(&self.filename)?;
        let header = reader
            .inner
            .read_header()
            .expect("Failed to read a BAM header");

        let batch_size = self.batch_size.unwrap_or(Self::DEFAULT_BATCH_SIZE);

        let inflags = self.inflags.unwrap_or(0);
        let exflags = self.exflags.unwrap_or(2564);
        let minmapq = self.minmapq.unwrap_or(0);

        Ok(Reader::new(
            self.filename,
            reader,
            header,
            None,
            batch_size,
            inflags,
            exflags,
            minmapq,
        ))
    }
}
