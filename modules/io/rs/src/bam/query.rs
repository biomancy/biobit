use std::io;

use gat_lending_iterator::LendingIterator;
use noodles::{bgzf, csi};
use noodles::bam::{io::Reader, Record};
use noodles::core::region::Interval;
use noodles::csi::binning_index::index::reference_sequence::bin::Chunk;
use noodles::sam::alignment::Record as _;

pub struct Query<'a, R> {
    reader: Reader<csi::io::Query<'a, R>>,
    reference_sequence_id: usize,
    interval: Interval,
    cache: &'a mut Vec<Record>,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
}

impl<'a, R> Query<'a, R>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    pub fn new(
        reader: &'a mut R,
        chunks: Vec<Chunk>,
        reference_sequence_id: usize,
        interval: Interval,
        cache: &'a mut Vec<Record>,
        inflags: u16,
        exflags: u16,
        minmapq: u8,
    ) -> Self {
        Self {
            reader: Reader::from(csi::io::Query::new(reader, chunks)),
            reference_sequence_id,
            interval,
            cache,
            inflags,
            exflags,
            minmapq,
        }
    }

    fn is_record_ok(&self, record: &Record) -> io::Result<bool> {
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
                let alignment_interval = (start..=end).into();
                Ok(id == self.reference_sequence_id && self.interval.intersects(alignment_interval))
            }
            _ => Ok(false),
        }
    }


    fn read(&mut self) -> io::Result<usize> {
        let mut processed = 0;
        while processed < self.cache.len() {
            // Try to read a record into the cache, if it fails, break the loop
            if self.reader.read_record(&mut self.cache[processed])? == 0 {
                break;
            }

            // Check if the record intersects with the target region => move to the next record
            if self.is_record_ok(&self.cache[processed])? {
                processed += 1;
            }
        }
        Ok(processed)
    }
}


impl<'a, R> LendingIterator for Query<'a, R>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    type Item<'b>
    where
        Self: 'b,
    = io::Result<&'b [Record]>;

    fn next(self: &'_ mut Self) -> Option<Self::Item<'_>> {
        match self.read() {
            Ok(0) => None,
            Ok(n) => Some(Ok(&self.cache[..n])),
            Err(e) => Some(Err(e)),
        }
    }
}

fn a() {
    use super::reader::ReaderBuilder;
    let mut reader = ReaderBuilder::new("file.bam").build();
    let mut query = reader.query("chr1", 1, 100).unwrap();
    while let Some(records) = query.next() {
        let records = records.unwrap();
        for record in records {
            println!("{}", record.alignment_start().unwrap().unwrap());
        }
    }
}
