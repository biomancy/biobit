use std::fs::File;
use std::io;
use std::path::PathBuf;

use ::higher_kinded_types::prelude::*;
use derive_getters::{Dissolve, Getters};
use noodles::{bam, bgzf, sam};
use noodles::core::position::Position;
use noodles::core::region::{Interval, Region};
use noodles::csi::BinningIndex;

use biobit_core_rs::LendingIterator;

use super::{
    indexed_reader::IndexedReader,
    query::Query,
    traits::IndexedBAM,
};

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

        let batch_size = self.batch_size.unwrap_or_else(|| Self::DEFAULT_BATCH_SIZE);
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
            inflags: self.inflags.unwrap_or(0),
            exflags: self.exflags.unwrap_or(0),
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
    inflags: u16,
    exflags: u16,
    minmapq: u8,
}

impl IndexedBAM for Reader {
    type Idx = usize;
    type Ctg = String;
    type Item = For!(<'iter> = &'iter [bam::Record]);

    fn fetch<'borrow>(
        &'borrow mut self,
        contig: &Self::Ctg,
        start: Self::Idx,
        end: Self::Idx,
    ) -> io::Result<
        Box<
            dyn 'borrow
            + LendingIterator<Item=For!(<'iter> = io::Result<<Self::Item as ForLt>::Of<'iter>>)>,
        >,
    > {
        let region = Region::new(
            contig.clone(),
            Interval::from(
                Position::try_from(start + 1).unwrap()..=Position::try_from(end).unwrap(),
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

        Ok(Box::new(Query::new(
            self.inner.inner.get_mut(),
            chunks,
            reference_sequence_id,
            region.interval(),
            &mut self.buffer,
            self.inflags,
            self.exflags,
            self.minmapq,
        )))
    }

    fn cloned<'borrow>(
        &'borrow self,
    ) -> Box<dyn 'borrow + Sync + IndexedBAM<Idx=Self::Idx, Ctg=Self::Ctg, Item=Self::Item>>
    where
        Self: 'borrow,
    {
        Box::new((*self).clone())
    }
}

impl Clone for Reader {
    fn clone(&self) -> Self {
        ReaderBuilder {
            filename: self.filename.clone(),
            inflags: Some(self.inflags),
            exflags: Some(self.exflags),
            minmapq: Some(self.minmapq),
            buffer: None,
            batch_size: Some(self.buffer.capacity()),
        }
            .build()
            .unwrap()
    }
}
