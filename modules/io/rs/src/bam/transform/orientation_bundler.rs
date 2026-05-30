use std::io;

use derive_getters::Dissolve;
use higher_kinded_types::prelude::*;
use noodles::bam;

use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{Orientation, PerOrientation};
use biobit_core_rs::source::{AnyMap, Transform};

use crate::bam::strdeductor::StrDeductor;

const ORIENTATIONS: [Orientation; 3] = [
    Orientation::Forward,
    Orientation::Reverse,
    Orientation::Dual,
];

#[derive(Debug, Clone, Default, Dissolve)]
pub struct Cache {
    batches: PerOrientation<Vec<bam::Record>>,
}

impl Cache {
    pub fn clear(&mut self) {
        self.batches.apply(|_, batch| batch.clear());
    }
}

#[derive(Debug, Clone, Default)]
pub struct BundleByOrientation<D: StrDeductor> {
    cache: Option<Cache>,
    batch_size: usize,
    deductor: D,
}

impl<D: StrDeductor> BundleByOrientation<D> {
    pub const DEFAULT_BATCH_SIZE: usize = 1024;

    pub fn new(deductor: D) -> Self {
        Self {
            cache: None,
            batch_size: Self::DEFAULT_BATCH_SIZE,
            deductor,
        }
    }
}

impl<InIter, D> Transform<InIter> for BundleByOrientation<D>
where
    D: StrDeductor,
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<
            Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>),
        >,
    >,
{
    type Args = ();
    type OutIter = For!(<'borrow> = Iterator<'borrow, InIter::Of<'borrow>, D>);
    type InItem = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>);
    type OutItem = For!(<'iter> = io::Result<(Orientation, &'iter [bam::Record])>);

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

    #[allow(clippy::needless_lifetimes)]
    fn transform<'borrow, 'args>(
        &'borrow mut self,
        iterator: InIter::Of<'borrow>,
        _: &'args Self::Args,
    ) -> <Self::OutIter as ForLt>::Of<'borrow> {
        let cache = self.cache.get_or_insert_with(Cache::default);
        cache.clear();

        Iterator {
            iterator,
            cache,
            deductor: &mut self.deductor,
            next_orientation: ORIENTATIONS.len(),
        }
    }
}

pub struct Iterator<'borrow, InIter, D: StrDeductor> {
    iterator: InIter,
    cache: &'borrow mut Cache,
    deductor: &'borrow mut D,
    next_orientation: usize,
}

impl<InIter, D> Iterator<'_, InIter, D>
where
    D: StrDeductor,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>)>,
{
    fn read(&mut self) -> io::Result<usize> {
        self.cache.clear();
        self.next_orientation = 0;

        loop {
            let Some(records) = self.iterator.next() else {
                return Ok(0);
            };
            let records = records?;
            if records.is_empty() {
                continue;
            }

            let len = records.len();
            for record in records.drain(..) {
                let orientation = self.deductor.deduce(&record);
                self.cache.batches[orientation].push(record);
            }
            return Ok(len);
        }
    }
}

impl<InIter, D> LendingIterator for Iterator<'_, InIter, D>
where
    D: StrDeductor,
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>)>,
{
    type Item = For!(<'iter> = io::Result<(Orientation, &'iter [bam::Record])>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        loop {
            if self.next_orientation >= ORIENTATIONS.len() {
                match self.read() {
                    Ok(0) => return None,
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            let orientation = ORIENTATIONS[self.next_orientation];
            self.next_orientation += 1;

            if self.cache.batches[orientation].is_empty() {
                continue;
            }
            return Some(Ok((
                orientation,
                self.cache.batches[orientation].as_slice(),
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io::Cursor;
    use std::num::NonZero;

    use noodles::core::Position;
    use noodles::sam;
    use noodles::sam::alignment::record::cigar::Op;
    use noodles::sam::alignment::record::cigar::op::Kind;
    use noodles::sam::alignment::record::{Flags, MappingQuality};
    use noodles::sam::alignment::record_buf::{QualityScores, Sequence};
    use noodles::sam::header::record::value::{Map, map::ReferenceSequence};

    use biobit_core_rs::source::Transform;

    use crate::bam::strdeductor;

    use super::*;

    type TestBatches = For!(<'borrow> = Batches);

    struct Batches {
        batches: VecDeque<io::Result<Vec<bam::Record>>>,
        current: Vec<bam::Record>,
    }

    impl Batches {
        fn new(batches: Vec<io::Result<Vec<bam::Record>>>) -> Self {
            Self {
                batches: batches.into(),
                current: Vec::new(),
            }
        }
    }

    impl LendingIterator for Batches {
        type Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>);

        fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
            match self.batches.pop_front()? {
                Ok(batch) => {
                    self.current = batch;
                    Some(Ok(&mut self.current))
                }
                Err(err) => Some(Err(err)),
            }
        }
    }

    fn record(flags: Flags) -> io::Result<bam::Record> {
        let header = sam::Header::builder()
            .add_reference_sequence(
                "chr1",
                Map::<ReferenceSequence>::new(const { NonZero::new(1000).unwrap() }),
            )
            .build();
        let record = sam::alignment::RecordBuf::builder()
            .set_name("r0")
            .set_flags(flags)
            .set_reference_sequence_id(0)
            .set_alignment_start(Position::try_from(1).unwrap())
            .set_mapping_quality(MappingQuality::try_from(60).unwrap())
            .set_cigar(vec![Op::new(Kind::Match, 1)].into_iter().collect())
            .set_sequence(Sequence::from(b"A"))
            .set_quality_scores(QualityScores::from(vec![30]))
            .build();

        let mut writer = bam::io::Writer::from(Vec::new());
        sam::alignment::io::Write::write_alignment_record(&mut writer, &header, &record)?;
        let mut reader = bam::io::Reader::from(Cursor::new(writer.into_inner()));
        let mut record = bam::Record::default();
        assert!(reader.read_record(&mut record)? > 0);
        Ok(record)
    }

    fn next_batch<InIter, D>(
        iter: &mut Iterator<'_, InIter, D>,
    ) -> io::Result<Option<(Orientation, Vec<bool>)>>
    where
        D: StrDeductor,
        InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>)>,
    {
        match iter.next() {
            None => Ok(None),
            Some(result) => {
                let (orientation, records) = result?;
                let reversed = records
                    .iter()
                    .map(|record| record.flags().is_reverse_complemented())
                    .collect();
                Ok(Some((orientation, reversed)))
            }
        }
    }

    #[test]
    fn splits_single_batch_by_orientation() -> io::Result<()> {
        let source = Batches::new(vec![Ok(vec![
            record(Flags::empty())?,
            record(Flags::REVERSE_COMPLEMENTED)?,
            record(Flags::empty())?,
        ])]);
        let mut transform = BundleByOrientation::new(strdeductor::deduce::se::forward);
        let mut iter = <BundleByOrientation<_> as Transform<TestBatches>>::transform(
            &mut transform,
            source,
            &(),
        );

        assert_eq!(
            next_batch(&mut iter)?,
            Some((Orientation::Forward, vec![false, false]))
        );
        assert_eq!(
            next_batch(&mut iter)?,
            Some((Orientation::Reverse, vec![true]))
        );
        assert_eq!(next_batch(&mut iter)?, None);
        Ok(())
    }

    #[test]
    fn yields_dual_for_unstranded_records() -> io::Result<()> {
        let source = Batches::new(vec![Ok(vec![
            record(Flags::empty())?,
            record(Flags::REVERSE_COMPLEMENTED)?,
        ])]);
        let mut transform = BundleByOrientation::new(strdeductor::deduce::se::unstranded);
        let mut iter = <BundleByOrientation<_> as Transform<TestBatches>>::transform(
            &mut transform,
            source,
            &(),
        );

        assert_eq!(
            next_batch(&mut iter)?,
            Some((Orientation::Dual, vec![false, true]))
        );
        assert_eq!(next_batch(&mut iter)?, None);
        Ok(())
    }

    #[test]
    fn propagates_upstream_errors() {
        let source = Batches::new(vec![Err(io::Error::new(io::ErrorKind::Other, "boom"))]);
        let mut transform = BundleByOrientation::new(strdeductor::deduce::se::forward);
        let mut iter = <BundleByOrientation<_> as Transform<TestBatches>>::transform(
            &mut transform,
            source,
            &(),
        );

        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
    }
}
