#![allow(clippy::too_many_arguments)]
use std::fs::File;
use std::io;
use std::path::PathBuf;

use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use eyre::{Result, ensure, eyre};
use higher_kinded_types::prelude::*;
use noodles::core::Position;
use noodles::core::region::Interval;
use noodles::{bam, bgzf, sam};

use biobit_core_rs::source::{AnyMap, Core, Source};

use super::indexed_reader::IndexedReader;
use super::query::{Cache, Query};

/// An indexed BAM source queried with 0-based, half-open coordinates.
#[derive(Dissolve, Getters, Constructor)]
// derive_more::Constructor generates explicit field names.
#[allow(clippy::redundant_field_names)]
pub struct Reader {
    filename: PathBuf,
    inner: IndexedReader<bgzf::io::reader::Reader<File>>,
    header: sam::header::Header,
    cache: Option<Cache>,
    batch_size: usize,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
}

impl PartialEq for Reader {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.header == other.header
            && self.batch_size == other.batch_size
            && self.inflags == other.inflags
            && self.exflags == other.exflags
            && self.minmapq == other.minmapq
    }
}

impl Clone for Reader {
    fn clone(&self) -> Self {
        Self {
            filename: self.filename.clone(),
            // Each clone needs its own file cursor, but the immutable index is shared.
            inner: IndexedReader {
                inner: bam::io::Reader::new(
                    File::open(&self.filename)
                        .expect("Failed to reopen a previously opened BAM file"),
                ),
                index: self.inner.index.clone(),
            },
            header: self.header.clone(),
            cache: None,
            batch_size: self.batch_size,
            inflags: self.inflags,
            exflags: self.exflags,
            minmapq: self.minmapq,
        }
    }
}

impl Core for Reader {
    type Args = For!(<'fetch> = (&'fetch String, usize, usize));
    type Item = For!(<'iter> = io::Result<&'iter mut Vec<bam::Record>>);

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

impl Source for Reader {
    type Iter = For!(<'borrow> = Query<'borrow, bgzf::io::reader::Reader<File>>);

    #[allow(clippy::needless_lifetimes)]
    fn fetch<'borrow, 'args>(
        &'borrow mut self,
        args: <<Self as Core>::Args as ForLt>::Of<'args>,
    ) -> Result<<Self::Iter as ForLt>::Of<'borrow>> {
        ensure!(
            args.1 < args.2,
            "BAM query requires start < end, got [{}, {})",
            args.1,
            args.2
        );
        ensure!(
            self.batch_size > 0,
            "BAM batch size must be greater than zero"
        );
        let interval =
            Interval::from(Position::try_from(args.1 + 1)?..=Position::try_from(args.2)?);
        let reference_sequence_id = self
            .header
            .reference_sequences()
            .get_index_of(args.0.as_bytes())
            .ok_or_else(|| eyre!("Unknown BAM reference sequence: {}", args.0))?;
        let chunks = self.inner.index.query(reference_sequence_id, interval)?;

        let cache = self.cache.get_or_insert_with(Cache::default);

        Ok(Query::new(
            self.inner.inner.get_mut(),
            chunks,
            reference_sequence_id,
            interval,
            cache,
            self.batch_size,
            self.inflags,
            self.exflags,
            self.minmapq,
        ))
    }
}
