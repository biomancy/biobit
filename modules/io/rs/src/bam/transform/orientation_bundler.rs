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
