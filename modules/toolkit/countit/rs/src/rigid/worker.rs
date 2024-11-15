use ahash::HashMap;
use derive_getters::Dissolve;
pub use eyre::Result;
use higher_kinded_types::prelude::*;
use std::collections::hash_map::Entry;

use crate::result::{PartitionMetrics, ResolutionOutcomes};
use crate::rigid::{resolution, Partition};
use biobit_collections_rs::interval_tree::{overlap, ITree};
use biobit_core_rs::{
    loc::{Contig, IntervalOp},
    num::{Float, PrimInt},
    source::{AnyMap, Source},
    LendingIterator,
};
use biobit_io_rs::bam::SegmentedAlignment;

#[derive(Clone, Debug, Default)]
struct CountingResult<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    cnts: Vec<Cnts>,
    stats: PartitionMetrics<Ctg, Idx, Cnts>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> CountingResult<Ctg, Idx, Cnts> {
    fn reset(&mut self, new_size: usize) {
        self.cnts.clear();
        self.cnts.resize(new_size, Cnts::zero());
        self.stats = PartitionMetrics::default();
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
    overlaps: Vec<overlap::Elements<Idx, usize>>,

    // Resolution strategy
    resolution: Box<dyn resolution::Resolution<Idx, Cnts, Elt>>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> Default for Worker<Ctg, Idx, Cnts, Elt> {
    fn default() -> Self {
        Self {
            accumulator: HashMap::default(),
            cache: AnyMap::new(),
            buffer: Vec::new(),
            overlaps: Vec::new(),
            resolution: Box::new(resolution::AnyOverlap::default()),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float, Elt> Worker<Ctg, Idx, Cnts, Elt> {
    pub fn new(resolution: Box<dyn resolution::Resolution<Idx, Cnts, Elt>>) -> Self {
        Self {
            accumulator: HashMap::default(),
            cache: AnyMap::new(),
            buffer: Vec::new(),
            overlaps: Vec::new(),
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
        elts: &[Elt],
        srcind: usize,
        source: &mut Src,
        prtind: usize,
        partition: &Partition<Ctg, Idx>,
    ) -> Result<()>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
    {
        source.populate_caches(&mut self.cache);
        self.resolution.reset(elts, partition.eltinds());

        let launched_at = std::time::Instant::now();
        let mut outcomes = ResolutionOutcomes::default();

        // Run the counting
        {
            let mut iterator = source.fetch((
                partition.contig(),
                partition.interval().start(),
                partition.interval().end(),
            ))?;

            // Get the cache and initialize it if needed
            self.setup(srcind, prtind, partition.eltinds().len());
            let counts = self.accumulator.get_mut(&(srcind, prtind)).unwrap();

            while let Some(blocks) = iterator.next() {
                let blocks = blocks?;

                // Populate the overlap
                if self.overlaps.len() < blocks.len() {
                    self.overlaps
                        .resize(blocks.len(), overlap::Elements::default());
                }
                for ((segments, orientation, _), overlap) in
                    blocks.iter().zip(self.overlaps.iter_mut())
                {
                    overlap.clear(); // Clear the previous overlap
                    partition
                        .index()
                        .get(orientation)
                        .overlap_single_element(segments, overlap); // Set the new one
                }

                // Resolve the overlaps
                self.resolution.resolve(
                    blocks,
                    &mut self.overlaps[0..blocks.len()],
                    &mut counts.cnts,
                    &mut outcomes,
                );
            }

            // Save the statistics
            counts.stats = PartitionMetrics {
                contig: partition.contig().clone(),
                interval: partition.interval().as_interval(),
                time_s: launched_at.elapsed().as_secs_f64(),
                outcomes,
            };
        }
        source.release_caches(&mut self.cache);
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn aggregate<'a>(
        sources: usize,
        elements: usize,
        partitions: &[Partition<Ctg, Idx>],
        workers: impl Iterator<Item = &'a mut Self>,
    ) -> Vec<(Vec<Cnts>, Vec<PartitionMetrics<Ctg, Idx, Cnts>>)>
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
                let eltinds = partitions[*prtind].eltinds();
                for (i, cnt) in result.cnts.iter().enumerate() {
                    let global_index = eltinds[i];
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
