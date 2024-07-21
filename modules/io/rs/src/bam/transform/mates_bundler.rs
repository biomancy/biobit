use std::collections::HashMap;
use std::io;

use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
use noodles::bam;
use noodles::core::Position;
use noodles::sam::alignment::Record;

use biobit_core_rs::LendingIterator;
use biobit_core_rs::source::Transform;

#[derive(Debug, Clone, PartialEq, Dissolve)]
struct CachedRecord {
    record: bam::Record,
    // Self
    start: Position,
    end: Position,
    reference_sequence_id: usize,
    // Mate
    mate_start: Position,
    mate_reference_sequence_id: usize,
}

impl TryInto<CachedRecord> for bam::Record {
    type Error = io::Error;

    fn try_into(self) -> Result<CachedRecord, Self::Error> {
        let start = self.alignment_start().transpose()?.ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Alignment start must be present in the BAM file",
        ))?;
        let end = self.alignment_end().transpose()?.ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Alignment end must be present in the BAM file",
        ))?;
        let reference_sequence_id =
            self.reference_sequence_id()
                .transpose()?
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Reference sequence ID must be present in the BAM file",
                ))?;
        let mate_start = self
            .mate_alignment_start()
            .transpose()?
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Mate alignment start must be present in the BAM file",
            ))?;
        let mate_reference_sequence_id =
            self.mate_reference_sequence_id()
                .transpose()?
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Mate reference sequence ID must be present in the BAM file",
                ))?;

        Ok(CachedRecord {
            record: self,
            start,
            end,
            reference_sequence_id,
            mate_start,
            mate_reference_sequence_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default, Dissolve)]
struct Bundle {
    lmate: Vec<CachedRecord>,
    rmate: Vec<CachedRecord>,
}

impl Bundle {
    fn push(&mut self, record: bam::Record) -> io::Result<()> {
        if record.flags().is_first_segment() {
            self.lmate.push(record.try_into()?);
        } else {
            self.rmate.push(record.try_into()?);
        }

        Ok(())
    }

    fn try_bundle(&mut self, writeto: &mut Vec<(bam::Record, bam::Record)>) -> io::Result<usize> {
        let mut lmate = 0;
        let mut bunled = 0;

        while lmate < self.lmate.len() {
            let mut paired = false;

            for rmate in 0..self.rmate.len() {
                let (left, right) = (&self.lmate[lmate], &self.rmate[rmate]);
                if left.mate_reference_sequence_id == right.reference_sequence_id
                    && left.mate_start == right.start
                    && left.record.flags().is_mate_reverse_complemented()
                        == right.record.flags().is_reverse_complemented()
                    && left.record.flags().is_mate_unmapped() == right.record.flags().is_unmapped()
                    && right.mate_reference_sequence_id == left.reference_sequence_id
                    && right.mate_start == left.start
                    && right.record.flags().is_mate_reverse_complemented()
                        == left.record.flags().is_reverse_complemented()
                    && right.record.flags().is_mate_unmapped() == left.record.flags().is_unmapped()
                {
                    writeto.push((
                        self.lmate.remove(lmate).record,
                        self.rmate.remove(rmate).record,
                    ));
                    paired = true;
                    break;
                }
            }

            if !paired {
                lmate += 1;
            } else {
                bunled += 1;
            }
        }

        Ok(bunled)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Dissolve)]
pub struct Cache {
    batch: Vec<(bam::Record, bam::Record)>,
    arena: Vec<Bundle>,
    bundles: HashMap<Vec<u8>, Bundle>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BundleMates {
    bundle_every: usize,
    batch_size: usize,
}

impl Default for BundleMates {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BUNDLE_EVERY, Self::DEFAULT_BATCH_SIZE)
    }
}

impl BundleMates {
    pub const DEFAULT_BUNDLE_EVERY: usize = 16;
    pub const DEFAULT_BATCH_SIZE: usize = 1024;

    pub fn new(bundle_every: usize, batch_size: usize) -> Self {
        Self {
            bundle_every,
            batch_size,
        }
    }

    fn try_bundle(&self, cache: &mut Cache) -> io::Result<()> {
        cache.bundles = std::mem::take(&mut cache.bundles)
            .into_iter()
            .filter_map(
                |(qname, mut bundle)| match bundle.try_bundle(&mut cache.batch) {
                    Ok(0) => {
                        cache.arena.push(bundle);
                        None
                    }
                    Ok(_) => Some(Ok((qname, bundle))),
                    Err(err) => Some(Err(err)),
                },
            )
            .collect::<io::Result<_>>()?;
        Ok(())
    }

    fn read(
        &self,
        iterator: &mut impl LendingIterator<Item = For!(<'iter> = io::Result<&'iter [bam::Record]>)>,
        cache: &mut Cache,
    ) -> io::Result<usize> {
        let mut consumed = 0;
        cache.batch.clear();

        while let Some(batch) = iterator.next() {
            for record in batch? {
                let qname = record.name().ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Query name must be present in the BAM file",
                ))?;

                if cache.bundles.contains_key(qname.as_bytes()) {
                    let bundle = cache.bundles.get_mut(qname.as_bytes()).unwrap();
                    bundle.push(record.clone())?;
                } else {
                    let mut bundle = cache.arena.pop().unwrap_or_default();
                    bundle.push(record.clone())?;
                    cache.bundles.insert(qname.as_bytes().to_vec(), bundle);
                }

                consumed += 1;
            }

            // Run the bundling if needed
            if consumed >= self.bundle_every {
                self.try_bundle(cache)?;
                consumed -= self.bundle_every;
            }

            if cache.batch.len() >= self.batch_size {
                break;
            }
        }

        if cache.batch.len() < self.batch_size && !cache.bundles.is_empty() {
            self.try_bundle(cache)?;
        }

        Ok(cache.batch.len())
    }
}

impl<InIter> Transform<InIter> for BundleMates
where
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<Item = For!(<'iter> = io::Result<&'iter [bam::Record]>)>,
    >,
{
    type Args = ();
    type Cache = Cache;
    type OutIter = For!(<'borrow> = Iterator<'borrow, InIter::Of<'borrow>>);
    type InItem = For!(<'iter> = io::Result<&'iter [bam::Record]>);
    type OutItem = For!(<'iter> = io::Result<&'iter [(bam::Record, bam::Record)]>);

    fn setup(&mut self, batch_size: usize, cache: &mut Self::Cache) {
        self.batch_size = batch_size;

        cache.batch.clear();

        for bndl in &mut cache.arena {
            bndl.lmate.clear();
            bndl.rmate.clear();
        }

        for (_, mut bndl) in cache.bundles.drain() {
            bndl.lmate.clear();
            bndl.rmate.clear();

            cache.arena.push(bndl)
        }

        self.batch_size = batch_size;
    }

    fn transform<'borrow>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'borrow Self::Args,
        cache: &'borrow mut Self::Cache,
    ) -> <Self::OutIter as ForLt>::Of<'borrow> {
        Iterator {
            iterator,
            cache,
            bundler: self,
        }
    }
}

pub struct Iterator<'borrow, InIter> {
    iterator: InIter,
    cache: &'borrow mut Cache,
    bundler: &'borrow mut BundleMates,
}

impl<'borrow, InIter> LendingIterator for Iterator<'borrow, InIter>
where
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter [bam::Record]>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter [(bam::Record, bam::Record)]>);

    fn next(&'_ mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.bundler.read(&mut self.iterator, self.cache) {
            Ok(0) => None,
            Ok(_) => Some(Ok(&self.cache.batch)),
            Err(err) => Some(Err(err)),
        }
    }
}
