use std::io;

use derive_more::{From, Into};
use higher_kinded_types::prelude::*;
use noodles::sam::alignment::Record;
use noodles::{
    bam, bam::io::Reader, bgzf, core::region::Interval, csi,
    csi::binning_index::index::reference_sequence::bin::Chunk,
};

use biobit_core_rs::LendingIterator;

#[derive(From, Into, Default)]
pub struct Cache {
    buffer: bam::Record,
    batch: Vec<bam::Record>,
}

pub struct Query<'a, R> {
    reader: Reader<csi::io::Query<'a, R>>,
    reference_sequence_id: usize,
    interval: Interval,
    cache: &'a mut Cache,
    batch_size: usize,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
}

impl<'a, R> Query<'a, R>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reader: &'a mut R,
        chunks: Vec<Chunk>,
        reference_sequence_id: usize,
        interval: Interval,
        cache: &'a mut Cache,
        batch_size: usize,
        inflags: u16,
        exflags: u16,
        minmapq: u8,
    ) -> Self {
        Self {
            reader: Reader::from(csi::io::Query::new(reader, chunks)),
            reference_sequence_id,
            interval,
            cache,
            batch_size,
            inflags,
            exflags,
            minmapq,
        }
    }

    fn is_record_ok(&self, record: &bam::Record) -> io::Result<bool> {
        let flags: u16 = record.flags().into();
        let mapq = record.mapping_quality().map(|x| x.get()).unwrap_or(255);
        let flags_ok = flags & self.inflags == self.inflags
            && flags & self.exflags == 0
            && mapq >= self.minmapq;
        if !flags_ok {
            return Ok(false);
        }

        match (
            record.reference_sequence_id().transpose()?,
            record.alignment_start().transpose()?,
            record.alignment_end().transpose()?,
        ) {
            (Some(id), Some(start), Some(end)) => {
                let interval = Interval::from(start..=end);
                Ok(id == self.reference_sequence_id && self.interval.intersects(interval))
            }
            _ => Ok(false),
        }
    }

    fn read(&mut self) -> io::Result<usize> {
        self.cache.batch.clear();

        while self.cache.batch.len() < self.batch_size {
            // Try to read a record into the cache, if it fails, break the loop
            if self.reader.read_record(&mut self.cache.buffer)? == 0 {
                break;
            }

            // Check if the record intersects with the target region => move to the next record
            if self.is_record_ok(&self.cache.buffer)? {
                let record = std::mem::take(&mut self.cache.buffer);
                self.cache.batch.push(record);
            }
        }
        Ok(self.cache.batch.len())
    }
}

impl<'borrow, R> LendingIterator for Query<'borrow, R>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    type Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>);

    fn next(&'_ mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.read() {
            Ok(0) => None,
            Ok(_) => Some(Ok(&mut self.cache.batch)),
            Err(e) => Some(Err(e)),
        }
    }
}
