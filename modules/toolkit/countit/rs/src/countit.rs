use std::cell::RefCell;
use std::io;

use higher_kinded_types::prelude::*;
use rayon::ThreadPool;
use thread_local::ThreadLocal;

use biobit_collections_rs::genomic_index::{GenomicIndex, OverlapSegments, OverlapSteps};
use biobit_collections_rs::interval_tree;
use biobit_core_rs::{
    LendingIterator,
    loc::{Contig, Locus, LocusLike, SegmentLike},
    num::{Float, PrimInt},
};
use biobit_io_rs::bam::{AlignmentSegments, IndexedBAM};

use super::result;

type Idx = usize;

#[derive(Clone, Debug)]
struct ThreadCache<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    initialized: bool,
    cnts: Vec<Cnts>,
    stats: Vec<result::Stats<Cnts, Ctg, Idx>>,
}

pub struct CountIt<Data, Cnts, Ctg, IT>
where
    Cnts: Float + Send,
    Ctg: Contig + Send,
    IT: interval_tree::ITree<Idx=Idx, Value=usize>,
{
    pool: ThreadPool,
    data: Vec<Data>,
    index: GenomicIndex<Ctg, IT>,
    // Thread local storage per source item = Vec<cnts per object + stats per partition>
    // Cnts per partition are summed per source
    cache: ThreadLocal<RefCell<Vec<ThreadCache<Ctg, Idx, Cnts>>>>,
}

impl<Data, Cnts, Ctg, IT> CountIt<Data, Cnts, Ctg, IT>
where
        for<'a> &'a Ctg: Send,
        Data: Clone + Send,
        Cnts: Float + Send,
        Ctg: Contig + Send,
        IT: interval_tree::ITree<Idx=Idx, Value=usize> + Send,
        for<'a> &'a Self: Send,
        GenomicIndex<Ctg, IT>: Send,
{
    pub fn new(thread_pool: ThreadPool, data: Vec<Data>, index: GenomicIndex<Ctg, IT>) -> Self {
        Self {
            pool: thread_pool,
            data,
            index,
            cache: ThreadLocal::new(),
        }
    }

    fn calculate<Source>(
        &self,
        ind: usize,
        source: &mut Source,
        partition: Locus<Ctg, Idx>,
    ) -> io::Result<()>
    where
        Source: IndexedBAM<
            Idx=Idx,
            Ctg=Ctg,
            Item=For!(<'iter> = &'iter AlignmentSegments<usize>),
        >,
        for<'borrow> interval_tree::ITreeIter<'borrow, IT>: LendingIterator,
        for<'borrow, 'iter> interval_tree::ITreeItem<'borrow, 'iter, IT>:
        interval_tree::TreeRecord<'borrow, 'iter, Idx=Idx, Value=usize>,
    {
        // Get/create the cache for the current thread
        let mut cache = self.cache.get_or(|| RefCell::new(Vec::new())).borrow_mut();
        if cache.len() <= ind {
            cache.resize(
                ind + 1,
                ThreadCache {
                    initialized: false,
                    cnts: Vec::new(),
                    stats: Vec::new(),
                },
            )
        }
        let cache = &mut cache[ind];

        // Initialize the cache if needed
        if !cache.initialized {
            cache.stats.clear();

            cache.cnts.clear();
            cache.cnts.resize(self.data.len(), Cnts::zero());
        }

        // Run the counting
        let launched_at = std::time::Instant::now();
        let (mut inside_annotation, mut outside_annotation) = (Cnts::zero(), Cnts::zero());

        let mut source = source
            .fetch(
                partition.contig(),
                partition.segment().start(),
                partition.segment().end(),
            )
            .unwrap();
        let mut overlaps = OverlapSegments::new();
        let mut steps = OverlapSteps::new();

        while let Some(blocks) = source.next() {
            for (segments, orientation) in blocks?.iter() {
                overlaps =
                    self.index
                        .overlap(partition.contig(), orientation, segments.iter(), overlaps);
                steps.build(segments.iter().zip(overlaps.iter()));

                let length: Idx = segments.iter().map(|x| x.len()).fold(0, |sum, x| sum + x);
                let length = Cnts::from(length).unwrap();

                for segment_steps in steps.iter() {
                    for (start, end, hits) in segment_steps {
                        let weight = Cnts::from(end - start).unwrap() / length;

                        if hits.is_empty() {
                            outside_annotation = outside_annotation + weight;
                        } else {
                            inside_annotation = inside_annotation + weight;
                            for x in hits {
                                cache.cnts[***x] = cache.cnts[***x] + weight;
                            }
                        }
                    }
                }
            }
        }

        cache.stats.push(result::Stats::new(
            partition.contig.clone(),
            partition.segment.clone(),
            launched_at.elapsed().as_secs_f64(),
            inside_annotation,
            outside_annotation,
        ));
        Ok(())
    }

    pub fn count<Source, Loc>(
        &mut self,
        sources: Vec<Source>,
        ids: Vec<String>,
        partitions: &[Loc],
    ) -> Vec<result::Counts<Data, Cnts, Ctg, Idx>>
    where
        Loc: Sync + LocusLike<Idx=Idx, Contig=Ctg>,
        Source: Sync
        + IndexedBAM<Idx=Idx, Ctg=Ctg, Item=For!(<'iter> = &'iter AlignmentSegments<usize>)>,
        for<'borrow> interval_tree::ITreeIter<'borrow, IT>: LendingIterator,
        for<'borrow, 'iter> interval_tree::ITreeItem<'borrow, 'iter, IT>:
        interval_tree::TreeRecord<'borrow, 'iter, Idx=Idx, Value=usize>,
    {
        // Soft reset all caches
        for cache in self.cache.iter_mut() {
            let cache = cache.get_mut();
            for element in cache {
                element.initialized = false;
            }
        }

        // Reuse any internal caches per thread later
        let thread_local_sources: ThreadLocal<
            RefCell<
                Vec<
                    Box<
                        dyn Sync
                        + IndexedBAM<
                            Idx=Idx,
                            Ctg=Ctg,
                            Item=For!(<'iter> = &'iter AlignmentSegments<usize>),
                        >,
                    >,
                >,
            >,
        > = ThreadLocal::new();

        // Run the counting
        let inds = (0..sources.len()).collect::<Vec<_>>();
        self.pool.scope(|s| {
            for ind in &inds {
                for partition in partitions {
                    s.spawn(|_| {
                        let mut sources = thread_local_sources
                            .get_or(|| RefCell::new(sources.iter().map(|x| x.cloned()).collect()))
                            .borrow_mut();

                        self.calculate(*ind, &mut sources[*ind], partition.as_locus()).expect(
                            "Failed to calculate counts for a partition. This is a bug.",
                        );
                    });
                }
            }
        });

        // Collapse the final results
        let mut results: Vec<_> = ids
            .into_iter()
            .map(|id| {
                (
                    id,
                    vec![Cnts::zero(); self.data.len()],
                    Vec::with_capacity(partitions.len()),
                )
            })
            .collect();

        for cache in self.cache.iter_mut() {
            for (ind, source_cache) in cache.get_mut().iter_mut().enumerate() {
                if source_cache.initialized {
                    results[ind].2.append(&mut source_cache.stats);
                    for (i, cnt) in source_cache.cnts.iter().enumerate() {
                        results[ind].1[i] = results[ind].1[i] + *cnt;
                    }
                }
            }
        }

        let results = results
            .into_iter()
            .map(|(id, cnts, stats)| result::Counts::new(id, self.data.clone(), cnts, stats))
            .collect();

        return results;
    }
}
