use ahash::HashMap;
use derive_getters::Dissolve;
pub use eyre::Result;
use ::higher_kinded_types::prelude::*;
use std::collections::hash_map::Entry;

use crate::result::{ResolutionOutcome, Summary};
use crate::rigid::{resolution, Partition};
use crate::Counts;
use biobit_collections_rs::interval_tree::overlap;
use biobit_collections_rs::interval_tree::ITree;
use biobit_core_rs::loc::Orientation;
use biobit_core_rs::{
    loc::{AsLocus, AsSegment, Contig},
    num::{Float, PrimInt},
    source::{AnyMap, Source},
    LendingIterator,
};
use biobit_io_rs::bam::SegmentedAlignment;

#[derive(Clone, Debug, Default)]
struct CountingResult<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    cnts: Vec<Cnts>,
    stats: Summary<Ctg, Idx>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> CountingResult<Ctg, Idx, Cnts> {
    fn reset(&mut self, new_size: usize) {
        self.cnts.clear();
        self.cnts.resize(new_size, Cnts::zero());
        self.stats = Summary::default();
    }
}

#[derive(Dissolve)]
pub struct Worker<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> {
    // (Source id, partition id) -> Cache entry
    accumulator: HashMap<(usize, usize), CountingResult<Ctg, Idx, Cnts>>,

    // Core caching for incoming alignment sources
    cache: AnyMap,
    // Stupid cache workaround because I can't store non 'static values in the AnyMap
    buffer: Vec<CountingResult<Ctg, Idx, Cnts>>,

    // Cache for the overlap calculation
    overlaps: overlap::Elements<Idx, usize>,

    // Resolution strategy
    resolution: Box<dyn resolution::Resolution<Idx, Cnts, Elt>>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> Default for Worker<Ctg, Idx, Cnts, Elt> {
    fn default() -> Self {
        Self {
            accumulator: HashMap::default(),
            cache: AnyMap::new(),
            buffer: Vec::new(),
            overlaps: overlap::Elements::default(),
            resolution: Box::new(resolution::Binary::new()),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> Worker<Ctg, Idx, Cnts, Elt> {
    pub fn new(resolution: Box<dyn resolution::Resolution<Idx, Cnts, Elt>>) -> Self {
        Self {
            accumulator: HashMap::default(),
            cache: AnyMap::new(),
            buffer: Vec::new(),
            overlaps: overlap::Elements::default(),
            resolution,
        }
    }

    pub fn reset(&mut self, resolution: Box<dyn resolution::Resolution<Idx, Cnts, Elt>>) {
        for entry in self.accumulator.drain() {
            self.buffer.push(entry.1);
        }
        self.resolution = resolution;
    }

    pub fn clear(&mut self) {
        self.accumulator.clear();
        self.accumulator.shrink_to_fit();

        self.cache.clear();
        self.cache.shrink_to_fit();
    }

    fn setup(&mut self, srcind: usize, prtind: usize, size: usize) {
        let mut cnts = self.buffer.pop().unwrap_or_default();
        cnts.reset(size);

        match self.accumulator.entry((srcind, prtind)) {
            Entry::Occupied(_) => panic!(
                "Worker cache already contains an entry for source {srcind} and partition {prtind}"
            ),
            Entry::Vacant(entry) => entry.insert(cnts),
        };
    }

    pub fn process<Src>(
        &mut self,
        srcind: usize,
        source: &mut Src,
        prtind: usize,
        partition: &Partition<Ctg, Idx, Elt>,
    ) -> Result<()>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        source.populate_caches(&mut self.cache);
        self.resolution.reset(partition.elements());

        let launched_at = std::time::Instant::now();
        let mut outcomes = ResolutionOutcome::default();

        // Run the counting
        {
            let mut iterator = source.fetch((
                partition.contig(),
                partition.segment().start(),
                partition.segment().end(),
            ))?;

            // Get the cache and initialize it if needed
            self.setup(srcind, prtind, partition.elements().len());
            let counts = self.accumulator.get_mut(&(srcind, prtind)).unwrap();

            while let Some(blocks) = iterator.next() {
                let blocks = blocks?;

                // Populate the overlap
                self.overlaps.reset();
                for (segments, orientation, _) in blocks.iter() {
                    partition
                        .index()
                        .get(orientation)
                        .overlap_single_element(segments, &mut self.overlaps);
                }

                // Resolve the overlaps
                self.resolution.resolve(
                    blocks,
                    &mut self.overlaps,
                    partition.elements(),
                    &mut counts.cnts,
                    &mut outcomes,
                );
            }

            // Save the statistics
            counts.stats = Summary::new(
                partition.contig().clone(),
                partition.segment().as_segment(),
                launched_at.elapsed().as_secs_f64(),
                outcomes,
            );
        }
        source.release_caches(&mut self.cache);
        Ok(())
    }

    pub fn aggregate<'a>(
        sources: usize,
        elements: usize,
        partitions: &[Partition<Ctg, Idx, Elt>],
        workers: impl Iterator<Item = &'a mut Self>,
    ) -> Vec<(Vec<Cnts>, Vec<Summary<Ctg, Idx>>)>
    where
        Ctg: 'a,
        Idx: 'a,
        Cnts: 'a,
        Elt: 'a,
    {
        let mut collapsed: Vec<_> = (0..sources)
            .map(|_| (vec![Cnts::zero(); elements], vec![None; partitions.len()]))
            .collect();

        for worker in workers {
            for ((srcind, prtind), result) in worker.accumulator.iter() {
                let saveto = &mut collapsed[*srcind];

                // Save the statistics
                debug_assert!(saveto.1[*prtind].is_none());
                saveto.1[*prtind] = Some(result.stats.clone());

                // Add the counts from the cache to the collapsed counts
                let ordering = partitions[*prtind].ordering();
                for (i, cnt) in result.cnts.iter().enumerate() {
                    let global_index = ordering[i];
                    saveto.0[global_index] = saveto.0[global_index] + *cnt;
                }
            }
        }

        // Unwrap all statistics
        collapsed.into_iter().map(|(cnts, stats)| {
            let stats = stats.into_iter()
                .map(|x| x.expect(
                    "Failed to populate counting statistic vector. This is a bug, please fill an issue."
                )).collect();
            (cnts, stats)
        }).collect()
    }
}
