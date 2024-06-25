use std::collections::HashMap;
use std::hash::Hash;

use rayon::ThreadPool;

use biobit_core_rs::loc::Contig;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::traits::reads_counter;
use by_address::ByThinAddress;
use biobit_core_rs::traits::reads_counter::Partition;

use super::result;

//  _sources: dict[_K, Source] = field(alias="sources")
//  resolution: Resolution[list[Overlap[_T]], Counts[_T | None]]
//  parallel: Parallel
//  _counts: dict[_K, Counts[_T]] = field(factory=dict, init=False)
//  _stats: list[CountingStats] = field(factory=list, init=False)

// pub struct Stats<Ctg: Contig, Idx: PrimInt> {
//     time_s: f64,
//     partition: Locus<Ctg, Idx>,
//     inside_annotation: f64,
//     outside_annotation: f64,
// }

// pub struct Counts<'a, Src, Data, Cnts: Float> {
//     source: &'a Src,
//     counts: &'a HashMap<Data, Cnts>,
// }

struct CountIt<Source, Data, Cnts: Float, Ctg: Contig, Idx: PrimInt> {
    pool: ThreadPool,
    counts: Vec<result::Counts<Source, Data, Cnts, Ctg, Idx>>,
}


impl<Source, Data, Cnts, Ctg, Idx> reads_counter::Counter for CountIt<Source, Data, Cnts, Ctg, Idx>
where
    Source: Send,
    Data: Hash + PartialEq + Send,
    Cnts: Float + Send,
    Ctg: Contig + Send,
    Idx: PrimInt + Send,
{
    type Ctg = Ctg;
    type Idx = Idx;
    type Source = Source;
    type Data = Data;
    type Result = result::Counts<Source, Data, Cnts, Ctg, Idx>;

    fn count(&mut self, partition: Vec<Partition<Self::Ctg, Self::Idx, Self::Data>>) {
        // let mut results = Vec::new();

        // let cnts = self.pool.scope(|s| {
        //     for p in partition {
        //         // for c in &self.counts {
        //         let mut r = HashMap::<ByThinAddress<&Self::Data>, Cnts>::new();
        //         // results.push(HashMap::<&Self::Data, Cnts>::new());
        //         //
        //         // let r = &mut results[results.len() - 1];
        //         s.spawn(|_| {
        //             r.clear();
        //         })
        //         // }
        //     }
        // });

        // Parallelism is per partition - per source pair
        // And I don't count as is, I count using references and then merge at the end

        todo!()
    }

    fn results(&self) -> &[Self::Result] { &self.counts }

    fn reset(&mut self) -> Vec<Self::Result> { std::mem::take(&mut self.counts) }
}

// import copy
// import time
// from collections import defaultdict
// from collections.abc import Iterable
// from typing import TypeVar, cast
//
// from attrs import define
// from attrs import field
// from intervaltree import IntervalTree
// from joblib import Parallel, delayed
//
// from biobit.core import Interval, Orientation
// from biobit.ds.gindex import Overlap, GenomicIndex
// from .partition import Partition
// from .reads_counter import MultiReadsCounter, CountingStats
// from ..resolve import Counts, Resolution
// from ..source import Source
//
// _T = TypeVar('_T')
// _K = TypeVar('_K')
//
//
// def run(
//         tag: _K,
//         source: Source,
//         partition: Partition[_T],
//         resolution: Resolution[list[Overlap[_T]], Counts[_T]],
// ) -> tuple[_K, Counts[_T], CountingStats]:
//     launched_at = time.time()
//     counts: dict[_T | None, float] = defaultdict(float)
//
//     if len(partition) == 0:
//         for _ in source.fetch(partition.contig, partition.rng.start, partition.rng.end):
//             for k, v in resolution([]).items():
//                 counts[k] += v
//     else:
//         # Build the index
//         skeleton = {
//             (partition.contig, Orientation.fwd): IntervalTree(),
//             (partition.contig, Orientation.rev): IntervalTree(),
//             (partition.contig, Orientation.dual): IntervalTree(),
//         }
//         assert all(i.contig == partition.contig for i in partition.intervals)
//
//         start, end = partition.rng.start, partition.rng.end
//         for interval, data in zip(partition.intervals, partition.data):
//             skeleton[(partition.contig, interval.orient)].addi(
//                 interval.rng.start, interval.rng.end, data=data
//             )
//             start = min(start, interval.rng.start)
//             end = max(end, interval.rng.end)
//
//         # Sanity check
//         if start != partition.rng.start or end != partition.rng.end:
//             raise ValueError(
//                 f"Partition intervals do not cover the entire partition range: "
//                 f"expected {partition.rng}, but got {(start, end)}"
//             )
//
//         index: GenomicIndex[_T] = GenomicIndex(skeleton)
//
//         # Count reads
//         for blocks in source.fetch(partition.contig, partition.rng.start, partition.rng.end):
//             overlaps = [index.overlap(partition.contig, blocks.orientation, rng=rng) for rng in blocks.blocks]
//             for k, v in resolution(overlaps).items():
//                 counts[k] += v
//
//     finished_at = time.time()
//
//     no_overlap = counts.pop(None, 0)
//     no_null_counts = cast(dict[_T, float], dict(counts))
//
//     stats = CountingStats(
//         time=finished_at - launched_at,
//         partition=Interval(partition.contig, partition.rng),
//         inside=sum(counts.values()),
//         outside=no_overlap,
//         extra=source.stats()
//     )
//
//     return tag, no_null_counts, stats
//
//
// @define(slots=True)
// class JoblibMultiReadsCounter(MultiReadsCounter[_T, _K]):
//     """
//     A class representing a reads counter that uses joblib to parallelize the counting process.
//     """
//     _sources: dict[_K, Source] = field(alias="sources")
//     resolution: Resolution[list[Overlap[_T]], Counts[_T | None]]
//     parallel: Parallel
//     _counts: dict[_K, Counts[_T]] = field(factory=dict, init=False)
//     _stats: list[CountingStats] = field(factory=list, init=False)
//
//     def count(self, partitions: Iterable[Partition]) -> None:
//         """
//         Count the reads in the given intervals.
//
//         :param partitions: Partitions to count reads in.
//         """
//         # Run the counting process
//         results = self.parallel(
//             delayed(run)(tag, copy.deepcopy(source), partition, self.resolution)
//             for partition in partitions
//             for tag, source in self._sources.items()
//         )
//
//         # Merge counts & stats
//         for tag, counts, stats in results:
//             _counts = self._counts.setdefault(tag, {})
//             for k, v in counts.items():
//                 _counts[k] = _counts.get(k, 0) + v
//
//             self._stats.append(stats)
//
//     def counts(self) -> dict[_K, Counts[_T]]:
//         return self._counts
//
//     def sources(self) -> dict[_K, Source]:
//         return self._sources
//
//     def stats(self) -> Iterable[CountingStats]:
//         return self._stats
//
//     def reset(self):
//         self._counts.clear()
//         self._stats.clear()