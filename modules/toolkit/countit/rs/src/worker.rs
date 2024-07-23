use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;
pub use eyre::Result;

use biobit_collections_rs::genomic_index::{GenomicIndex, OverlapSegments, OverlapSteps};
use biobit_collections_rs::interval_tree::ITree;
use biobit_core_rs::{
    LendingIterator,
    loc::{AsLocus, AsSegment, Contig},
    num::{Float, PrimInt},
    source::{AnyMap, Source},
};
use biobit_io_rs::bam::SegmentedAlignment;

use super::result::Stats;

#[derive(Clone, Debug)]
struct Cache<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    initialized: bool,
    cnts: Vec<Cnts>,
    stats: Vec<Stats<Ctg, Idx, Cnts>>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Cache<Ctg, Idx, Cnts> {
    fn reset(&mut self) {
        self.initialized = false;
    }

    fn initialize(&mut self, objects: usize) {
        if self.initialized {
            debug_assert_eq!(self.cnts.len(), objects);
            return;
        }

        self.stats.clear();

        self.cnts.clear();
        self.cnts.resize(objects, Cnts::zero());

        self.initialized = true;
    }
}

#[derive(Debug, Dissolve)]
pub struct Worker<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // There is a clear one-to-one correspondence between Cache entries and Sources being processed.
    // Each entry serve as an accumulator for the counts and stats calculated for each partition in
    // each Source.
    cache: Vec<Cache<Ctg, Idx, Cnts>>,
    srcache: AnyMap,

    // Cache for the overlap calculations
    overlap_segments: OverlapSegments<'static, Idx, usize>,
    overlap_steps: OverlapSteps<'static, Idx, usize>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Default for Worker<Ctg, Idx, Cnts> {
    fn default() -> Self {
        Self {
            cache: Vec::default(),
            srcache: AnyMap::new(),
            overlap_segments: OverlapSegments::default(),
            overlap_steps: OverlapSteps::default(),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Worker<Ctg, Idx, Cnts> {
    pub fn reset(&mut self) {
        // Soft reset to avoid memory reallocation
        for entry in self.cache.iter_mut() {
            entry.reset();
        }
    }

    fn cache_for(&mut self, srcind: usize, objects: usize) -> &mut Cache<Ctg, Idx, Cnts> {
        if srcind >= self.cache.len() {
            self.cache.resize_with(srcind + 1, || Cache {
                initialized: false,
                cnts: Vec::new(),
                stats: Vec::new(),
            });
        }

        self.cache[srcind].initialize(objects);
        &mut self.cache[srcind]
    }

    pub fn calculate<Src, IT, Lcs>(
        &mut self,
        ind: usize,
        objects: usize,
        index: &GenomicIndex<Ctg, IT>,
        source: &mut Src,
        partition: &Lcs,
    ) -> Result<()>
    where
        Src: Source<
            Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
            Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
        >,
        Lcs: AsLocus<Contig = Ctg, Idx = Idx>,
        IT: ITree<Idx = Idx, Value = usize>,
    {
        source.populate_caches(&mut self.srcache);
        let launched_at = std::time::Instant::now();
        let (mut inside_annotation, mut outside_annotation) = (Cnts::zero(), Cnts::zero());

        {
            let mut iterator = source.fetch((
                partition.contig(),
                partition.segment().start(),
                partition.segment().end(),
            ))?;

            let mut overlaps = std::mem::take(&mut self.overlap_segments);
            let mut steps = std::mem::take(&mut self.overlap_steps);

            // Run the counting
            {
                // Get the cache and initialize it if needed
                let cache = self.cache_for(ind, objects);

                while let Some(blocks) = iterator.next() {
                    for (segments, orientation, n) in blocks?.iter() {
                        overlaps = index.overlap(
                            partition.contig(),
                            orientation,
                            segments.iter(),
                            overlaps,
                        );
                        debug_assert!(overlaps.len() == segments.len());
                        steps.build(segments.iter().zip(overlaps.iter()));

                        let length: Idx = segments
                            .iter()
                            .map(|x| x.len())
                            .fold(Idx::zero(), |sum, x| sum + x);

                        let weight =
                            Cnts::one() / (Cnts::from(length).unwrap() * Cnts::from(n).unwrap());

                        for segment_steps in steps.iter() {
                            for (start, end, hits) in segment_steps {
                                let segweight = Cnts::from(end - start).unwrap() * weight;

                                // consumed = consumed + weight;
                                if hits.is_empty() {
                                    outside_annotation = outside_annotation + segweight;
                                } else {
                                    inside_annotation = inside_annotation + segweight;
                                    let segweight = segweight / Cnts::from(hits.len()).unwrap();
                                    for x in hits {
                                        cache.cnts[***x] = cache.cnts[***x] + segweight;
                                    }
                                }
                            }
                        }
                        // debug_assert!(
                        //     (consumed.to_f64().unwrap() - 1.0).abs() < 1e-6,
                        //     "Consumed: {:?}",
                        //     consumed.to_f64().unwrap()
                        // );
                    }
                }

                cache.stats.push(Stats::new(
                    partition.contig().clone(),
                    partition.segment().as_segment(),
                    launched_at.elapsed().as_secs_f64(),
                    inside_annotation,
                    outside_annotation,
                ));
            }

            // Save back the overlap cache
            self.overlap_segments = overlaps.reset();
            self.overlap_steps = steps.reset();
        }
        source.release_caches(&mut self.srcache);
        Ok(())
    }

    pub fn collapse<'a>(
        sources: usize,
        objects: usize,
        partitions: usize,
        workers: impl Iterator<Item = &'a mut Worker<Ctg, Idx, Cnts>>,
    ) -> Vec<(Vec<Cnts>, Vec<Stats<Ctg, Idx, Cnts>>)>
    where
        Idx: 'a,
        Cnts: 'a,
        Ctg: 'a,
    {
        let mut collapsed: Vec<_> = (0..sources)
            .map(|_| (vec![Cnts::zero(); objects], Vec::with_capacity(partitions)))
            .collect();

        for worker in workers {
            // Ensure that the cache is not larger than the number of sources
            debug_assert!(worker.cache.len() <= sources);

            for (srcind, cache) in worker.cache.iter_mut().enumerate() {
                if !cache.initialized {
                    // Cache has not been initialized, skip (this source wasn't processed in this thread)
                    continue;
                }

                let saveto = &mut collapsed[srcind];

                // Append the stats from the cache to the collapsed stats
                saveto.1.append(&mut cache.stats);

                // Add the counts from the cache to the collapsed counts
                debug_assert_eq!(objects, cache.cnts.len());
                for (i, cnt) in cache.cnts.iter().enumerate() {
                    saveto.0[i] = saveto.0[i] + *cnt;
                }
            }
        }

        debug_assert!(collapsed.iter().all(|(_, stats)| stats.len() == partitions));
        collapsed
    }
}
