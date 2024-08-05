use std::io;

use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
use dyn_clone::DynClone;
use noodles::bam::record::Record;
use noodles::sam::alignment::record::cigar::Op;
use noodles::sam::alignment::record::cigar::op::Kind;
use noodles::sam::alignment::record::data::field::{Tag, Value};

use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{Orientation, Segment};
use biobit_core_rs::num::PrimInt;
use biobit_core_rs::source::{AnyMap, Transform};

use crate::bam::{alignment_segments::AlignmentSegments, strdeductor::StrDeductor};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Dissolve)]
pub struct SegmentedAlignment<Idx: PrimInt> {
    pub segments: AlignmentSegments<Idx>,
    pub orientation: Vec<Orientation>,
    pub total_hits: Vec<i8>,
}

impl<Idx: PrimInt> SegmentedAlignment<Idx> {
    pub fn clear(&mut self) {
        self.segments.clear();
        self.orientation.clear();
        self.total_hits.clear();
    }

    pub fn push(&mut self, segments: &[Segment<Idx>], orientation: Orientation, total_hits: i8) {
        if segments.is_empty() {
            return;
        }

        self.segments.push(segments);
        self.orientation.push(orientation);
        self.total_hits.push(total_hits);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[Segment<Idx>], Orientation, i8)> {
        self.segments
            .iter()
            .zip(&self.orientation)
            .zip(&self.total_hits)
            .map(|((segments, orientation), total_hits)| (segments, *orientation, *total_hits))
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Dissolve)]
pub struct Cache {
    segments: Vec<Segment<usize>>,
    batch: SegmentedAlignment<usize>,
}

impl Cache {
    fn append_cigar(
        &mut self,
        mut start: usize,
        cigar: impl Iterator<Item = io::Result<Op>>,
    ) -> io::Result<&mut Self> {
        self.segments.clear();

        for cigar in cigar {
            let cigar = cigar?;
            let len = cigar.len();

            match cigar.kind() {
                Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                    let segment = Segment::new(start, start + len).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid CIGAR operation {:?}", cigar),
                        )
                    })?;

                    self.segments.push(segment);
                    start += len;
                }
                Kind::Deletion | Kind::Skip => {
                    // Deletion or skip don't produce an 'aligned' segment in the read
                    start += len;
                }
                Kind::Insertion | Kind::SoftClip | Kind::HardClip | Kind::Pad => {}
            }
        }
        Ok(self)
    }
}

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

    fn transform<'borrow, 'args>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'args Self::Args,
    ) -> <Self::OutIter as ForLt>::Of<'borrow> {
        let cache = self.cache.get_or_insert_with(|| Cache::default());
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

impl<'borrow, InIter, D> LendingIterator for AlnSegmentsIterator<'borrow, InIter, D>
where
    D: StrDeductor + Clone,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<Record>>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.iterator.next()? {
            Ok(records) => {
                self.cache.batch.clear();

                for record in records {
                    let orientation = self.deductor.deduce(record);

                    // Reconstruct the alignment segments from the record
                    let start = record.alignment_start()?.ok()?.get();
                    self.cache.segments.clear();
                    self.cache.append_cigar(start, record.cigar().iter()).ok()?;

                    let total_hits = extract_aln_hit_count(record).unwrap();
                    self.cache
                        .batch
                        .push(&self.cache.segments, orientation, total_hits);
                }
                Some(Ok(&mut self.cache.batch))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct ExtractPairedAlignmentSegments<D: StrDeductor> {
    inner: ExtractAlignmentSegments<D>,
}

impl<D: StrDeductor + DynClone> ExtractPairedAlignmentSegments<D> {
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

    fn transform<'borrow, 'args>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'args Self::Args,
    ) -> <Self::OutIter as ForLt>::Of<'borrow> {
        let cache = self.inner.cache.get_or_insert_with(|| Cache::default());
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

impl<'borrow, InIter, D> LendingIterator for PairedAlnSegmentsIterator<'borrow, InIter, D>
where
    D: StrDeductor + Clone,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<(Record, Record)>>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<usize>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.iterator.next()? {
            Ok(records) => {
                self.cache.batch.clear();

                for (lmate, rmate) in records {
                    // Predict the orientation
                    let lorientation = self.deductor.deduce(lmate);
                    let rorientation = self.deductor.deduce(rmate);

                    if lorientation != rorientation {
                        return Some(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Inconsistent orientation predicted for: {:?} and {:?} ({} vs {})",
                                lmate, rmate, lorientation, rorientation
                            ),
                        )));
                    }

                    // Reconstruct the alignment segments from the record
                    self.cache.segments.clear();
                    self.cache
                        .append_cigar(
                            lmate.alignment_start()?.ok()?.get() - 1,
                            lmate.cigar().iter(),
                        )
                        .ok()?
                        .append_cigar(
                            rmate.alignment_start()?.ok()?.get() - 1,
                            rmate.cigar().iter(),
                        )
                        .ok()?;
                    self.cache.segments = Segment::merge(&mut self.cache.segments);

                    let lhits = extract_aln_hit_count(lmate).unwrap();
                    debug_assert!(lhits > 0);
                    debug_assert!(lhits == extract_aln_hit_count(rmate).unwrap());

                    self.cache
                        .batch
                        .push(&self.cache.segments, lorientation, lhits);
                }
                Some(Ok(&mut self.cache.batch))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub fn extract_aln_hit_count(record: &Record) -> io::Result<i8> {
    let data = record.data();
    let total_hits = match data.get(&Tag::ALIGNMENT_HIT_COUNT) {
        Some(Ok(tag)) => tag,
        Some(Err(e)) => return Err(e),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "No TOTAL_HIT_COUNT tag",
            ))
        }
    };
    match total_hits {
        Value::Int8(tag) => Ok(tag),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TOTAL_HIT_COUNT tag must be an Int32",
            ))
        }
    }
}
