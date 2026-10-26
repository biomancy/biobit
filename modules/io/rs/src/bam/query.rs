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

        // Check the reference and start before walking CIGAR to calculate the end.
        if record.reference_sequence_id().transpose()? != Some(self.reference_sequence_id) {
            return Ok(false);
        }
        let Some(start) = record.alignment_start().transpose()? else {
            return Ok(false);
        };
        if self.interval.end().is_some_and(|end| start > end) {
            return Ok(false);
        }

        // Earlier starts may still overlap the query, so their end must be checked.
        match record.alignment_end().transpose()? {
            Some(end) => {
                let interval = Interval::from(start..=end);
                Ok(self.interval.intersects(interval))
            }
            None => Ok(false),
        }
    }

    fn read(&mut self) -> io::Result<usize> {
        let mut count = 0;
        while count < self.batch_size {
            // Reuse record allocations when consumers borrow rather than drain a batch.
            if count == self.cache.batch.len() {
                self.cache.batch.push(bam::Record::default());
            }
            if self.reader.read_record(&mut self.cache.batch[count])? == 0 {
                break;
            }
            if self.is_record_ok(&self.cache.batch[count])? {
                count += 1;
            }
        }
        self.cache.batch.truncate(count);
        Ok(count)
    }
}

impl<R> LendingIterator for Query<'_, R>
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
