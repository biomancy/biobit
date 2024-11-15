use std::io;

use derive_getters::Dissolve;
use higher_kinded_types::prelude::*;
use noodles::bam;

use biobit_core_rs::source::{AnyMap, Transform};
use biobit_core_rs::LendingIterator;
use bundle::Bundler;

mod bundle;
#[derive(Debug, Clone, Default, Dissolve)]
pub struct Cache {
    batch: Vec<(bam::Record, bam::Record)>,
    bundler: Bundler,
}

impl Cache {
    pub fn clear(&mut self) {
        self.batch.clear();
        self.bundler.clear();
    }
}

#[derive(Debug, Clone)]
pub struct BundleMates {
    cache: Option<Cache>,
    batch_size: usize,
}

impl Default for BundleMates {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BATCH_SIZE)
    }
}

impl BundleMates {
    pub const DEFAULT_BATCH_SIZE: usize = 1024;

    pub fn new(batch_size: usize) -> Self {
        Self {
            cache: None,
            batch_size,
        }
    }
}

impl<InIter> Transform<InIter> for BundleMates
where
    InIter: for<'borrow> ForLt<
        Of<'borrow>: LendingIterator<
            Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>),
        >,
    >,
{
    type Args = ();
    type OutIter = For!(<'borrow> = Iterator<'borrow, InIter::Of<'borrow>>);
    type InItem = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>);
    type OutItem = For!(<'iter> = io::Result<&'iter mut Vec<(bam::Record, bam::Record)>>);

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
    ) -> <Self::OutIter as ForLifetime>::Of<'borrow> {
        let cache = self.cache.get_or_insert_with(Default::default);
        cache.clear();

        Iterator {
            iterator,
            batch_size: self.batch_size,
            cache,
        }
    }
}

pub struct Iterator<'borrow, InIter> {
    iterator: InIter,
    batch_size: usize,
    cache: &'borrow mut Cache,
}

impl<'borrow, InIter> Iterator<'borrow, InIter>
where
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>)>,
{
    fn read(&mut self) -> io::Result<usize> {
        self.cache.batch.clear();
        while let Some(batch) = self.iterator.next() {
            let batch = batch?;
            for record in batch.drain(..) {
                match self.cache.bundler.push(record)? {
                    Some((a, b)) => {
                        self.cache.batch.push((a, b));
                    }
                    None => continue,
                }
            }

            if self.cache.batch.len() >= self.batch_size {
                break;
            }
        }
        Ok(self.cache.batch.len())
    }
}

impl<'borrow, InIter> LendingIterator for Iterator<'borrow, InIter>
where
    InIter: LendingIterator<Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>)>,
{
    type Item = For!(<'iter> = io::Result<&'iter mut Vec<(bam::Record, bam::Record)>>);

    fn next(&'_ mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.read() {
            Ok(0) => None,
            Ok(_) => Some(Ok(&mut self.cache.batch)),
            Err(err) => Some(Err(err)),
        }
    }
}
