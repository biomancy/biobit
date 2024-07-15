use std::fs::File;
use std::io;
use std::path::PathBuf;

use ::higher_kinded_types::prelude::*;
use derive_getters::{Dissolve, Getters};
use eyre::Result;
use noodles::{bam, bgzf, sam};
use noodles::core::{Position, Region};
use noodles::core::region::Interval;
use noodles::csi::BinningIndex;

use biobit_core_rs::source;

use super::indexed_reader::IndexedReader;
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

    pub fn build(self) -> io::Result<Reader> {
        let mut reader = IndexedReader::new(&self.filename)?;
        let header = reader
            .inner
            .read_header()
            .expect("Failed to read a BAM header");

        let batch_size = self.batch_size.unwrap_or(Self::DEFAULT_BATCH_SIZE);
        let mut buffer = self
            .buffer
            .unwrap_or_else(|| Vec::with_capacity(batch_size));

        // Clear the buffer and resize it to the batch size
        buffer.clear();
        buffer.resize(batch_size, bam::Record::default());

        Ok(Reader {
            filename: self.filename,
            inner: reader,
            header,
            buffer,
            batch_size,
            inflags: self.inflags.unwrap_or(0),
            exflags: self.exflags.unwrap_or(516),
            minmapq: self.minmapq.unwrap_or(0),
        })
    }
}

#[derive(Dissolve, Getters)]
pub struct Reader {
    filename: PathBuf,
    inner: IndexedReader<bgzf::reader::Reader<File>>,
    header: sam::header::Header,
    buffer: Vec<bam::Record>,
    batch_size: usize,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
}

impl PartialEq for Reader {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.header == other.header
            && self.batch_size == other.batch_size
            && self.inflags == other.inflags
            && self.exflags == other.exflags
            && self.minmapq == other.minmapq
    }
}

impl Clone for Reader {
    fn clone(&self) -> Self {
        Self {
            filename: self.filename.clone(),
            inner: IndexedReader::new(&self.filename).expect(
                "Failed to open a BAM file; \
                Note: the file had been opened before at least once without any errors.",
            ),
            header: self.header.clone(),
            buffer: vec![bam::Record::default(); self.batch_size],
            batch_size: self.batch_size,
            inflags: self.inflags,
            exflags: self.exflags,
            minmapq: self.minmapq,
        }
    }
}

impl source::Core for Reader {
    type Args = For!(<'fetch> = (&'fetch String, usize, usize));
    type Item = For!(<'iter> = io::Result<&'iter [bam::Record]>);

    fn batch_size(&self) -> usize {
        self.batch_size
    }

    fn with_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size;
    }
}

impl source::Source for Reader {
    type Iter = For!(<'borrow> = Query<'borrow, bgzf::reader::Reader<File>>);

    fn fetch<'borrow>(
        &'borrow mut self,
        args: <<Self as source::Core>::Args as ForLt>::Of<'_>,
    ) -> Result<<Self::Iter as ForLt>::Of<'borrow>> {
        let region = Region::new(
            args.0.clone(),
            Interval::from(
                Position::try_from(args.1 + 1).unwrap()..=Position::try_from(args.2).unwrap(),
            ),
        );

        let reference_sequence_id = self
            .header
            .reference_sequences()
            .get_index_of(region.name())
            .expect("Invalid reference sequence name");
        let chunks = self
            .inner
            .index
            .query(reference_sequence_id, region.interval())?;

        Ok(Query::new(
            self.inner.inner.get_mut(),
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
