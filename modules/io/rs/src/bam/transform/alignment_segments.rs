use std::io;

use derive_getters::Dissolve;
use higher_kinded_types::prelude::*;
use noodles::bam::record::Record;
use noodles::sam::alignment::record::cigar::Op;
use noodles::sam::alignment::record::cigar::op::Kind;
use noodles::sam::alignment::record::data::field::Tag;

use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{Interval, IntervalOp, Orientation};
use biobit_core_rs::num::PrimInt;
use biobit_core_rs::source::{AnyMap, Transform};

use crate::bam::{alignment_segments::AlignmentSegments, strdeductor::StrDeductor};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Dissolve)]
pub struct SegmentedAlignment<Idx: PrimInt> {
    pub intervals: AlignmentSegments<Idx>,
    pub orientation: Vec<Orientation>,
    pub total_hits: Vec<u32>,
}

impl<Idx: PrimInt> SegmentedAlignment<Idx> {
    pub fn clear(&mut self) {
        self.intervals.clear();
        self.orientation.clear();
        self.total_hits.clear();
    }

    pub fn push(&mut self, segments: &[Interval<Idx>], orientation: Orientation, total_hits: u32) {
        if segments.is_empty() {
            return;
        }

        self.intervals.push(segments);
        self.orientation.push(orientation);
        self.total_hits.push(total_hits);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[Interval<Idx>], Orientation, u32)> {
        self.intervals
            .iter()
            .zip(&self.orientation)
            .zip(&self.total_hits)
            .map(|((segments, orientation), total_hits)| (segments, *orientation, *total_hits))
    }

    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Dissolve)]
pub struct Cache {
    segments: Vec<Interval<usize>>,
    batch: SegmentedAlignment<usize>,
}

impl Cache {
    fn append_record(&mut self, record: &Record) -> io::Result<()> {
        let start = record.alignment_start().transpose()?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "BAM alignment has no start position",
            )
        })?;
        // Noodles exposes 1-based positions, while our intervals are 0-based.
        self.append_cigar(start.get() - 1, record.cigar().iter())?;
        Ok(())
    }

    fn append_cigar(
        &mut self,
        mut start: usize,
        cigar: impl Iterator<Item = io::Result<Op>>,
    ) -> io::Result<&mut Self> {
        for cigar in cigar {
            let cigar = cigar?;
            let len = cigar.len();

            match cigar.kind() {
                Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                    let end = start.checked_add(len).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "CIGAR reference position overflow",
                        )
                    })?;
                    let interval = Interval::new(start, end).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid CIGAR operation {:?}", cigar),
                        )
                    })?;

                    if let Some(last) = self.segments.last_mut()
                        && last.end() == start
                    {
                        *last = last
                            .union(&interval)
                            .expect("adjacent intervals have a union");
                    } else {
                        self.segments.push(interval);
                    }
                    start = end;
                }
                // Deletions and skips consume reference bases without aligning read bases.
                Kind::Deletion | Kind::Skip => {
                    start = start.checked_add(len).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "CIGAR reference position overflow",
                        )
                    })?;
                }
                Kind::Insertion | Kind::SoftClip | Kind::HardClip | Kind::Pad => {}
            }
        }
        Ok(self)
    }
}

/// Extracts aligned read bases as 0-based, half-open reference intervals.
/// Unmapped records are ignored; mapped records require a positive NH tag.
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct ExtractAlignmentSegments<D: StrDeductor> {
    batch_size: usize,
    cache: Option<Cache>,
    deductor: D,
}

impl<D: StrDeductor> ExtractAlignmentSegments<D> {
    pub fn new(deductor: D) -> Self {
        Self {
            batch_size: 1024,
            cache: None,
            deductor,
        }
    }
    fn populate_caches(&mut self, cache: &mut AnyMap) {
        let cache = cache.remove().unwrap_or_default();
        self.cache = Some(cache);
    }

    fn release_caches(&mut self, cache: &mut AnyMap) {
        match self.cache.take() {
            None => {}
            Some(x) => {
                cache.insert(x);
            }
        }
    }

    fn batch_size(&self) -> usize {
        self.batch_size
    }

    fn with_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size;
    }
}

impl<InIter, D> Transform<InIter> for ExtractAlignmentSegments<D>
where
    D: StrDeductor,
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<Record>>)>,
    >,
{
    type Args = ();
    type OutIter = For!(<'borrow> = AlnSegmentsIterator<'borrow, InIter::Of<'borrow>, D>);
    type InItem = For!(<'iter> = io::Result<&'iter mut Vec<Record>>);
    type OutItem = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);
    fn populate_caches(&mut self, cache: &mut AnyMap) {
        Self::populate_caches(self, cache);
    }

    fn release_caches(&mut self, cache: &mut AnyMap) {
        Self::release_caches(self, cache)
    }

    fn batch_size(&self) -> usize {
        Self::batch_size(self)
    }

    fn with_batch_size(&mut self, batch_size: usize) {
        Self::with_batch_size(self, batch_size)
    }

    #[allow(clippy::needless_lifetimes)]
    fn transform<'borrow, 'args>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'args Self::Args,
    ) -> <Self::OutIter as ForLt>::Of<'borrow> {
        let cache = self.cache.get_or_insert_with(Cache::default);
        AlnSegmentsIterator {
            iterator,
            deductor: &mut self.deductor,
            cache,
        }
    }
}

pub struct AlnSegmentsIterator<'borrow, InIter, D: StrDeductor> {
    iterator: InIter,
    deductor: &'borrow mut D,
    cache: &'borrow mut Cache,
}

impl<InIter, D> LendingIterator for AlnSegmentsIterator<'_, InIter, D>
where
    D: StrDeductor,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<Record>>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        self.iterator.next().map(|records| {
            let records = records?;
            self.cache.batch.clear();
            for record in records {
                if record.flags().is_unmapped() {
                    continue;
                }
                self.cache.segments.clear();
                self.cache.append_record(record)?;
                let total_hits = extract_aln_hit_count(record)?;
                self.cache.batch.push(
                    &self.cache.segments,
                    self.deductor.deduce(record),
                    total_hits,
                );
            }
            Ok(&mut self.cache.batch)
        })
    }
}

/// Unions aligned bases from both mates on the same reference, counting overlaps once.
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct ExtractPairedAlignmentSegments<D: StrDeductor> {
    inner: ExtractAlignmentSegments<D>,
}

impl<D: StrDeductor> ExtractPairedAlignmentSegments<D> {
    pub fn new(deductor: D) -> Self {
        Self {
            inner: ExtractAlignmentSegments::new(deductor),
        }
    }
}

impl<InIter, D> Transform<InIter> for ExtractPairedAlignmentSegments<D>
where
    D: StrDeductor,
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<
            Item = For!(<'iter> = io::Result<&'iter mut Vec<(Record, Record)>>),
        >,
    >,
{
    type Args = ();
    type OutIter = For!(<'borrow> = PairedAlnSegmentsIterator<'borrow, InIter::Of<'borrow>, D>);
    type InItem = For!(<'iter> = io::Result<&'iter mut Vec<(Record, Record)>>);
    type OutItem = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);

    fn populate_caches(&mut self, cache: &mut AnyMap) {
        self.inner.populate_caches(cache)
    }

    fn release_caches(&mut self, cache: &mut AnyMap) {
        self.inner.release_caches(cache)
    }

    fn batch_size(&self) -> usize {
        self.inner.batch_size()
    }

    fn with_batch_size(&mut self, batch_size: usize) {
        self.inner.with_batch_size(batch_size)
    }

    #[allow(clippy::needless_lifetimes)]
    fn transform<'borrow, 'args>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'args Self::Args,
    ) -> <Self::OutIter as ForLt>::Of<'borrow> {
        let cache = self.inner.cache.get_or_insert_with(Cache::default);
        PairedAlnSegmentsIterator {
            iterator,
            cache,
            deductor: &mut self.inner.deductor,
        }
    }
}
pub struct PairedAlnSegmentsIterator<'borrow, InIter, D: StrDeductor> {
    iterator: InIter,
    cache: &'borrow mut Cache,
    deductor: &'borrow mut D,
}

impl<InIter, D> LendingIterator for PairedAlnSegmentsIterator<'_, InIter, D>
where
    D: StrDeductor,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<(Record, Record)>>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        self.iterator.next().map(|records| {
            let records = records?;
            self.cache.batch.clear();
            for (lmate, rmate) in records {
                let reference = lmate.reference_sequence_id().transpose()?;
                if lmate.flags().is_unmapped()
                    || rmate.flags().is_unmapped()
                    || reference.is_none()
                    || reference != rmate.reference_sequence_id().transpose()?
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Paired alignment segments require two mapped mates on the same reference",
                    ));
                }
                let lorientation = self.deductor.deduce(lmate);
                let rorientation = self.deductor.deduce(rmate);
                if lorientation != rorientation {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Inconsistent orientation for BAM mates {:?}: {} vs {}",
                            lmate.name(),
                            lorientation,
                            rorientation
                        ),
                    ));
                }
                let hits = extract_aln_hit_count(lmate)?;
                if hits != extract_aln_hit_count(rmate)? {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "BAM mates have different NH alignment hit counts",
                    ));
                }

                self.cache.segments.clear();
                self.cache.append_record(lmate)?;
                self.cache.append_record(rmate)?;
                // Union the mates in place, retaining the scratch buffer between pairs.
                self.cache.segments.sort_unstable();
                self.cache.segments.dedup_by(|next, previous| {
                    if let Some(union) = previous.union(next) {
                        *previous = union;
                        true
                    } else {
                        false
                    }
                });
                self.cache
                    .batch
                    .push(&self.cache.segments, lorientation, hits);
            }
            Ok(&mut self.cache.batch)
        })
    }
}

pub fn extract_aln_hit_count(record: &Record) -> io::Result<u32> {
    record
        .data()
        .get(&Tag::ALIGNMENT_HIT_COUNT)
        .transpose()?
        .and_then(|value| value.as_int())
        .and_then(|value| u32::try_from(value).ok())
        .filter(|&value| value > 0)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "BAM NH alignment hit count must be present and be a positive integer",
            )
        })
}
