use std::ops::Range;
use crate::core::num::PrimUInt;
use derive_getters::{Dissolve, Getters};

// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Dissolve, Getters)]
// struct Partition<T, Idx: PrimUInt> {
//     contig: String,
//     partition: Range<Idx>,
//     // intervals: Vec<Interval>,
//     data: Vec<T>,
// }

// There should be multiple levels of genomic indexing
// ?? -> coordinates + orientation
// Interval -> coordinates + orientation + contig
// How to call ??. Could it be called a segment?


// Range -> coordinates
// Locus -> coordinates + contig (arbitrary type)

// Most of the time (?) I will be working with complete intervals. Right? Or not really? Yes.
// Or Ranges + contigs + orientation -> Locus?








// from collections import defaultdict
// from typing import TypeVar, Iterable, Generic
//
// from attrs import define
//
// from biobit import algo
// from biobit.core import Interval, Range
//
// _T = TypeVar('_T')
//
//
// @define(slots=True, frozen=True)
// class Partition(Generic[_T]):
//     """
//     A class representing a partition of intervals.
//
//     Attributes:
//         contig: The contig of the partition.
//         intervals: The intervals in the partition.
//         data: The data corresponding to the intervals.
//     """
//     contig: str
//     rng: Range
//     intervals: tuple[Interval, ...]
//     data: tuple[_T, ...]
//
//     def __attrs_post_init__(self):
//         if len(self.intervals) != len(self.data):
//             raise ValueError("Intervals and data lists must have the same length")
//
//     @staticmethod
//     def from_contigs(
//             data: Iterable[_T], intervals: Iterable[Interval], contig_size: dict[str, int]
//     ) -> list["Partition[_T]"]:
//         """
//         Create partitions from a list of intervals, grouping them by contigs.
//
//         The resulting partitions will cover the entire genome, with each partition
//         covering a single contig (even if there are no annotations on that contig).
//
//         :param data: An iterable of data objects.
//         :param intervals: An iterable of intervals associated with each data object.
//         :param contig_size: A dictionary mapping contig names to their sizes.
//         :return: A list of partitions.
//         """
//         # Group by contig
//         contigs = defaultdict(list)
//         for d, i in zip(data, intervals):
//             contigs[i.contig].append((i, d))
//
//         result: list[Partition[_T]] = []
//         # Add empty partitions for contigs without annotations
//         for contig, size in contig_size.items():
//             if contig not in contigs:
//                 result.append(Partition(contig, Range(0, size), tuple(), tuple()))
//
//         # Add partitions for contigs with annotations
//         for contig, elements in contigs.items():
//             if contig not in contig_size:
//                 raise ValueError(f"Contig {contig} not found in contig size dictionary")
//             start, end = 0, contig_size[contig]
//             intervals, data = zip(*elements)
//             result.append(Partition(contig, Range(start, end), tuple(intervals), tuple(data)))
//
//         return result
//
//     @staticmethod
//     def from_annotated(
//             data: Iterable[_T], intervals: Iterable[Interval], maxdist: int = 1024
//     ) -> list["Partition[_T]"]:
//         """
//         Create partitions from existing annotations, grouping them based on their contig and distance from each other.
//
//         Note, the resulting partitions will *not* cover the entire genome, only the regions where annotations are present.
//
//         :param data: An iterable of data objects.
//         :param intervals: An iterable of intervals associated with each data object.
//         :param maxdist: The maximum distance between intervals for them to be considered part of the same partition.
//         :return: A list of partitions.
//         """
//         # Group by contig
//         contigs = defaultdict(list)
//         for d, i in zip(data, intervals):
//             contigs[i.contig].append((i, d))
//
//         # Sort by start position and make final partitions
//         results = []
//         for contig, items in contigs.items():
//             items.sort(key=lambda x: x[0].rng.start)
//             groups = algo.misc.group_within(
//                 items,
//                 distance=lambda x, y: min(
//                     abs(y[0].rng.start - x[0].rng.start),
//                     abs(y[0].rng.end - x[0].rng.start),
//                     abs(y[0].rng.start - x[0].rng.end),
//                     abs(y[0].rng.end - x[0].rng.end)
//                 ),
//                 maxdist=maxdist
//             )
//
//             for grp in groups:
//                 pinterval, pdata = zip(*grp)
//                 start, end = min(i.rng.start for i in pinterval), max(i.rng.end for i in pinterval)
//                 results.append(Partition(contig, Range(start, end), pinterval, pdata))
//         return results
//
//     def __len__(self) -> int:
//         return len(self.intervals)