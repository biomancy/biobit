use std::io;

use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
use derive_more::{Constructor, From};
use noodles::bam::Record;
use noodles::sam::alignment::record::cigar::op::Kind;
use noodles::sam::alignment::record::cigar::Op;

use biobit_core_rs::loc::{Segment};
use biobit_core_rs::LendingIterator;

use crate::bam::{alignment_segments::AlignmentSegments, strdeductor::StrDeductor};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Dissolve)]
pub struct AlignmentSegmentAdapter<I, S> {
    inner: I,
    strander: S,
    cache: Vec<Segment<usize>>,
    alnsegments: AlignmentSegments<usize>,
}

impl<I, S> AlignmentSegmentAdapter<I, S> {
    pub const DEFAULT_CACHE_CAPACITY: usize = 32;

    pub fn new(inner: I, strander: S) -> Self {
        let cache = Vec::with_capacity(Self::DEFAULT_CACHE_CAPACITY);
        let alnsegments = AlignmentSegments::with_capacity(Self::DEFAULT_CACHE_CAPACITY);
        Self {
            inner,
            cache,
            alnsegments,
            strander,
        }
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

impl<I, S> LendingIterator for AlignmentSegmentAdapter<I, S>
where
    I: LendingIterator,
    for<'iter> <<I as LendingIterator>::Item as ForLt>::Of<'iter>:
        Into<io::Result<&'iter [Record]>>,
    S: StrDeductor,
{
    type Item = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>);

    fn next(self: &'_ mut Self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.inner.next()?.into() {
            Ok(records) => {
                self.alnsegments.clear();
                for record in records {
                    let orientation = self.strander.deduce(record);

                    // Reconstruct the alignment segments from the record
                    self.cache.clear();
                    Self::parse_cigar(
                        record.alignment_start()?.ok()?.get(),
                        record.cigar().iter(),
                        &mut self.cache,
                    )
                    .ok()?;

                    self.alnsegments.push(&self.cache, orientation);
                }
                Some(Ok(&self.alnsegments))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, From, Constructor, Dissolve)]
pub struct PairedEndAlignmentSegmentsAdapter<I, S>(AlignmentSegmentAdapter<I, S>);

impl<I, S> LendingIterator for PairedEndAlignmentSegmentsAdapter<I, S>
where
    I: LendingIterator,
    for<'iter> <<I as LendingIterator>::Item as ForLt>::Of<'iter>:
        Into<io::Result<&'iter [(Record, Record)]>>,
    S: StrDeductor,
{
    type Item = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>);

    fn next(self: &'_ mut Self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.0.inner.next()?.into() {
            Ok(records) => {
                self.0.alnsegments.clear();

                for (lmate, rmate) in records {
                    // Predict the orientation
                    let lorientation = self.0.strander.deduce(lmate);
                    let rorientation = self.0.strander.deduce(rmate);

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
                    self.0.cache.clear();
                    AlignmentSegmentAdapter::<I, S>::parse_cigar(
                        lmate.alignment_start()?.ok()?.get(),
                        lmate.cigar().iter(),
                        &mut self.0.cache,
                    )
                    .ok()?;
                    AlignmentSegmentAdapter::<I, S>::parse_cigar(
                        rmate.alignment_start()?.ok()?.get(),
                        rmate.cigar().iter(),
                        &mut self.0.cache,
                    )
                    .ok()?;
                    AlignmentSegmentAdapter::<I, S>::collapse(&mut self.0.cache);

                    self.0.alnsegments.push(&self.0.cache, lorientation);
                }
                Some(Ok(&self.0.alnsegments))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
