use std::ops::IndexMut;

use biobit_core_rs::num::PrimUInt;
use eyre::Result;
use noodles::bam::Record;
use noodles::bam::record::Sequence;
use noodles::sam::alignment::record::cigar::Op;
use noodles::sam::alignment::record::cigar::op::Kind;

use crate::dna::Observed;
use crate::pileup::Pileup;

#[derive(Debug)]
pub(super) struct CigarCursor<I> {
    op: Kind,
    len: usize,
    iter: I,
}

impl<I> CigarCursor<I> {
    #[inline]
    pub(super) fn op(&self) -> Kind {
        self.op
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(super) fn consume(&mut self, len: usize) {
        debug_assert!(
            len <= self.len,
            "consumed {len} CIGAR steps with only {} remaining",
            self.len
        );
        self.len -= len;
    }
}

impl<I, E> CigarCursor<I>
where
    I: Iterator<Item = std::result::Result<Op, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    #[inline]
    pub(super) fn new(mut iter: I) -> eyre::Result<Option<Self>> {
        let Some(op) = iter.next() else {
            return Ok(None);
        };
        let op = op?;
        let len = op.len();
        debug_assert!(len > 0, "CIGAR op has zero length");
        Ok(Some(Self {
            op: op.kind(),
            len,
            iter,
        }))
    }

    #[inline]
    pub(super) fn advance(&mut self) -> eyre::Result<bool> {
        let Some(op) = self.iter.next() else {
            return Ok(false);
        };
        let op = op?;
        self.op = op.kind();
        self.len = op.len();
        debug_assert!(self.len > 0, "CIGAR op has zero length");
        Ok(true)
    }
}

#[derive(Debug)]
pub(super) struct ReadCursor<'a> {
    sequence: Sequence<'a>,
    quality: &'a [u8],
    position: usize,
}

impl<'a> ReadCursor<'a> {
    #[inline]
    pub(super) fn new(record: &'a Record) -> Self {
        debug_assert!(
            record.sequence().len() == record.quality_scores().len(),
            "Record sequence and quality scores have different lengths"
        );
        Self {
            sequence: record.sequence(),
            quality: record.quality_scores().as_bytes(),
            position: 0,
        }
    }

    #[inline]
    pub(super) fn advance(&mut self, len: usize) {
        debug_assert!(
            self.position.checked_add(len).is_some(),
            "read cursor overflow while advancing by {len} steps"
        );
        self.position += len;
    }

    #[inline]
    pub(super) fn sequence(&self, offset: usize) -> Observed {
        // SAFETY: the caller must ensure that the offset is within the bounds of the sequence, which is guaranteed by the CIGAR parsing logic.
        Observed::from(unsafe { self.sequence.get(self.index(offset)).unwrap_unchecked() })
    }

    #[inline]
    pub(super) fn qual(&self, offset: usize) -> u8 {
        self.quality[self.index(offset)]
    }

    #[inline]
    fn index(&self, offset: usize) -> usize {
        debug_assert!(
            self.position.checked_add(offset).is_some(),
            "Read cursor index overflow while accessing offset {offset}"
        );
        debug_assert!(
            self.position + offset < self.quality.len(),
            "CIGAR consumes more query bases than record length"
        );
        self.position + offset
    }
}

#[derive(Debug)]
pub(super) struct RoiCursor<'a, Cnts: PrimUInt> {
    pileup: &'a mut Pileup<Cnts>,
    offset: usize,
}

impl<'a, Cnts: PrimUInt> RoiCursor<'a, Cnts> {
    #[inline]
    pub(super) fn new(pileup: &'a mut Pileup<Cnts>, offset: usize) -> Self {
        debug_assert!(
            offset <= pileup.len(),
            "ROI cursor offset {offset} is past pileup length {}",
            pileup.len()
        );
        Self { pileup, offset }
    }

    #[inline]
    pub(super) fn is_exhausted(&self) -> bool {
        self.offset == self.pileup.len()
    }

    #[inline]
    pub(super) fn remaining(&self) -> usize {
        self.pileup.len() - self.offset
    }

    #[inline]
    pub(super) fn advance(&mut self, len: usize) {
        debug_assert!(
            len <= self.remaining(),
            "advanced ROI cursor by {len} steps with only {} remaining",
            self.remaining()
        );
        self.offset += len;
    }

    #[inline]
    pub(super) fn deletion(&mut self, len: usize) -> Result<()> {
        debug_assert!(
            len <= self.remaining(),
            "deletion length {len} exceeds remaining ROI length {}",
            self.remaining()
        );

        for value in &mut self.pileup.deletion_mut()[self.offset..self.offset + len] {
            increment(value)?;
        }
        Ok(())
    }

    #[inline]
    pub(super) fn aligned(
        &mut self,
        read: &ReadCursor<'_>,
        len: usize,
        min_phread: u8,
    ) -> Result<()> {
        debug_assert!(
            len <= self.remaining(),
            "aligned length {len} exceeds remaining ROI length {}",
            self.remaining()
        );

        for i in 0..len {
            if read.qual(i) >= min_phread {
                let value = &mut self.pileup.index_mut(read.sequence(i))[self.offset + i];
                increment(value)?;
            }
        }
        Ok(())
    }
}

#[inline(always)]
fn increment<Cnts: PrimUInt>(value: &mut Cnts) -> Result<()> {
    *value = (*value)
        .checked_add(&Cnts::one())
        .ok_or_else(|| eyre::eyre!("Pileup count overflow"))?;
    Ok(())
}
