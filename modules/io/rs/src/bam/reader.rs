use std::fs::File;
use std::io;
use std::path::PathBuf;

use derive_getters::{Dissolve, Getters};
use noodles::{bam, bgzf, sam};
use noodles::core::position::Position;
use noodles::core::region::{Interval, Region};

use super::query::Query;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderBuilder {
    filename: PathBuf,
    inflags: Option<u16>,
    exflags: Option<u16>,
    minmapq: Option<u8>,
    buffer: Option<Vec<bam::Record>>,
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
            buffer: None,
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

    pub fn with_buffer(mut self, buffer: Vec<bam::Record>) -> Self {
        self.buffer = Some(buffer);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn build(self) -> Reader {
        let mut reader = bam::io::indexed_reader::Builder::default()
            .build_from_path(&self.filename)
            .expect("Failed to read a BAM file");
        let header = reader.read_header().expect("Failed to read a BAM header");

        let batch_size = self.batch_size.unwrap_or_else(|| Self::DEFAULT_BATCH_SIZE);
        let mut buffer = self.buffer.unwrap_or_else(|| Vec::with_capacity(batch_size));

        // Clear the buffer and resize it to the batch size
        buffer.clear();
        buffer.resize(batch_size, bam::Record::default());

        Reader {
            inner: reader,
            header,
            buffer,
            inflags: self.inflags.unwrap_or(0),
            exflags: self.exflags.unwrap_or(0),
            minmapq: self.minmapq.unwrap_or(0),
        }
    }
}

#[derive(Dissolve, Getters)]
pub struct Reader {
    inner: bam::io::IndexedReader<bgzf::reader::Reader<File>>,
    header: sam::header::Header,
    buffer: Vec<bam::Record>,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
}

impl Reader {
    pub fn query<'a, 'b>(
        &'a mut self, contig: &'b str, start: usize, end: usize,
    ) -> io::Result<Query<'a, bgzf::reader::Reader<File>>> {
        let region = Region::new(
            contig,
            Interval::from(
                Position::try_from(start + 1).unwrap()..=Position::try_from(end).unwrap(),
            ),
        );

        let reference_sequence_id = self.header.reference_sequences()
            .get_index_of(region.name())
            .expect("Invalid reference sequence name");
        let chunks = self.inner.index().query(reference_sequence_id, region.interval())?;

        Ok(Query::new(
            self.inner.get_mut(),
            chunks,
            reference_sequence_id,
            region.interval(),
            &mut self.buffer,
            self.inflags,
            self.exflags,
            self.minmapq,
        ))
    }
}
