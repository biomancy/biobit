use std::io;

use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
use dyn_clone::DynClone;
use noodles::bam::record::Record;
use noodles::sam::alignment::record::cigar::Op;
use noodles::sam::alignment::record::cigar::op::Kind;

use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::Segment;
use biobit_core_rs::source::Transform;

use crate::bam::{alignment_segments::AlignmentSegments, strdeductor::StrDeductor};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Dissolve)]
pub struct Cache {
    segments: Vec<Segment<usize>>,
    batch: AlignmentSegments<usize>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct ExtractAlignmentSegments<D: StrDeductor> {
    batch_size: usize,
    deductor: D,
}

impl<D: StrDeductor + DynClone> ExtractAlignmentSegments<D> {
    pub fn new(deductor: D) -> Self {
        Self {
            batch_size: 0,
            deductor,
        }
    }

    fn setup(&mut self, batch_size: usize, cache: &mut Cache) {
        cache.segments.clear();
        cache.batch.clear();

        self.batch_size = batch_size;
    }

    fn collapse(segments: &mut Vec<Segment<usize>>) {
        let mut writeto = 0;
        for pointer in 1..segments.len() {
            if let Some(union) = segments[pointer].union(&segments[writeto]) {
                segments[writeto] = union;
            } else {
                writeto += 1;
                segments[writeto] = segments[pointer];
            }
        }
        segments.truncate(writeto + 1);
    }

    fn parse_cigar(
        mut start: usize,
        cigar: impl Iterator<Item = io::Result<Op>>,
        saveto: &mut Vec<Segment<usize>>,
    ) -> io::Result<()> {
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

                    saveto.push(segment);
                    start += len;
                }
                Kind::Deletion | Kind::Skip => {
                    // Deletion or skip don't produce an 'aligned' segment in the read
                    start += len;
                }
                Kind::Insertion | Kind::SoftClip | Kind::HardClip | Kind::Pad => {}
            }
        }
        Ok(())
    }
}

pub struct AlnSegmentsIterator<'borrow, InIter, D: StrDeductor> {
    iterator: InIter,
    cache: &'borrow mut Cache,
    slf: &'borrow mut ExtractAlignmentSegments<D>,
}

impl<InIter, D> Transform<InIter> for ExtractAlignmentSegments<D>
where
    D: StrDeductor,
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<Item = For!(<'iter> = io::Result<&'iter [Record]>)>,
    >,
{
    type Args = ();
    type Cache = Cache;
    type OutIter = For!(<'borrow> = AlnSegmentsIterator<'borrow, InIter::Of<'borrow>, D>);
    type InItem = For!(<'iter> = io::Result<&'iter [Record]>);
    type OutItem = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>);

    fn setup(&mut self, batch_size: usize, cache: &mut Self::Cache) {
        ExtractAlignmentSegments::setup(self, batch_size, cache);
    }

    fn transform<'borrow>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'borrow Self::Args,
        cache: &'borrow mut Self::Cache,
    ) -> <Self::OutIter as ForLifetime>::Of<'borrow> {
        AlnSegmentsIterator {
            iterator,
            cache,
            slf: self,
        }
    }
}

impl<'borrow, InIter, D> LendingIterator for AlnSegmentsIterator<'borrow, InIter, D>
where
    D: StrDeductor + Clone,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter [Record]>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.iterator.next()? {
            Ok(records) => {
                self.cache.batch.clear();
                for record in records {
                    let orientation = self.slf.deductor.deduce(record);

                    // Reconstruct the alignment segments from the record
                    self.cache.segments.clear();
                    ExtractAlignmentSegments::<D>::parse_cigar(
                        record.alignment_start()?.ok()?.get(),
                        record.cigar().iter(),
                        &mut self.cache.segments,
                    )
                    .ok()?;

                    self.cache.batch.push(&self.cache.segments, orientation);
                }
                Some(Ok(&self.cache.batch))
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

pub struct PairedAlnSegmentsIterator<'borrow, InIter, D: StrDeductor> {
    iterator: InIter,
    cache: &'borrow mut Cache,
    slf: &'borrow mut ExtractAlignmentSegments<D>,
}

impl<InIter, D> Transform<InIter> for ExtractPairedAlignmentSegments<D>
where
    D: StrDeductor + Clone,
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<Item = For!(<'iter> = io::Result<&'iter [(Record, Record)]>)>,
    >,
{
    type Args = ();
    type Cache = Cache;
    type OutIter = For!(<'borrow> = PairedAlnSegmentsIterator<'borrow, InIter::Of<'borrow>, D>);
    type InItem = For!(<'iter> = io::Result<&'iter [(Record, Record)]>);
    type OutItem = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>);

    fn setup(&mut self, batch_size: usize, cache: &mut Self::Cache) {
        ExtractAlignmentSegments::setup(&mut self.inner, batch_size, cache);
    }

    fn transform<'borrow>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'borrow Self::Args,
        cache: &'borrow mut Self::Cache,
    ) -> <Self::OutIter as ForLifetime>::Of<'borrow> {
        PairedAlnSegmentsIterator {
            iterator,
            cache,
            slf: &mut self.inner,
        }
    }
}

impl<'borrow, InIter, D> LendingIterator for PairedAlnSegmentsIterator<'borrow, InIter, D>
where
    D: StrDeductor + Clone,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter [(Record, Record)]>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.iterator.next()? {
            Ok(records) => {
                self.cache.batch.clear();

                for (lmate, rmate) in records {
                    // Predict the orientation
                    let lorientation = self.slf.deductor.deduce(lmate);
                    let rorientation = self.slf.deductor.deduce(rmate);

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
                    ExtractAlignmentSegments::<D>::parse_cigar(
                        lmate.alignment_start()?.ok()?.get(),
                        lmate.cigar().iter(),
                        &mut self.cache.segments,
                    )
                    .ok()?;
                    ExtractAlignmentSegments::<D>::parse_cigar(
                        rmate.alignment_start()?.ok()?.get(),
                        rmate.cigar().iter(),
                        &mut self.cache.segments,
                    )
                    .ok()?;
                    ExtractAlignmentSegments::<D>::collapse(&mut self.cache.segments);

                    self.cache.batch.push(&self.cache.segments, lorientation);
                }
                Some(Ok(&self.cache.batch))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
