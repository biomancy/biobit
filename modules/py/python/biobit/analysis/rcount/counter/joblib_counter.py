import copy
import time
from attrs import define
from attrs import field
from collections import defaultdict
from collections.abc import Iterable
from intervaltree import IntervalTree
from joblib import Parallel, delayed
from typing import TypeVar, cast

from biobit.ds.gindex import Overlap, GenomicIndex
from biobit.core import Interval, Orientation, Range
from .partition import Partition
from .reads_counter import MultiReadsCounter, CountingStats
from ..resolve import Counts, Resolution
from ..source import Source

_T = TypeVar('_T')
_K = TypeVar('_K')


def run(
        tag: _K,
        source: Source,
        partition: Partition[_T],
        resolution: Resolution[list[Overlap[_T]], Counts[_T]],
) -> tuple[_K, Counts[_T], CountingStats]:
    if len(partition) == 0:
        raise ValueError("Partition must have at least one interval")

    launched_at = time.time()

    # Build the index
    skeleton = {
        (partition.contig, Orientation.fwd): IntervalTree(),
        (partition.contig, Orientation.rev): IntervalTree(),
        (partition.contig, Orientation.dual): IntervalTree(),
    }
    assert all(i.contig == partition.contig for i in partition.intervals)

    start, end = partition.intervals[0].rng.start, partition.intervals[0].rng.end
    for interval, data in zip(partition.intervals, partition.data):
        skeleton[(partition.contig, interval.orient)].addi(
            interval.rng.start, interval.rng.end, data=data
        )

        start = min(start, interval.rng.start)
        end = max(end, interval.rng.end)

    index: GenomicIndex[_T] = GenomicIndex(skeleton)
    index_range = Range(start, end)

    # Count reads
    counts: dict[_T | None, float] = defaultdict(float)
    for blocks in source.fetch(partition.contig, index_range.start, index_range.end):
        overlaps = [index.overlap(partition.contig, blocks.orientation, rng=rng) for rng in blocks.blocks]
        for k, v in resolution(overlaps).items():
            counts[k] += v

    finished_at = time.time()

    no_overlap = counts.pop(None, 0)
    no_null_counts = cast(dict[_T, float], dict(counts))

    stats = CountingStats(
        time=finished_at - launched_at,
        partition=Interval(partition.contig, index_range),
        inside=sum(counts.values()),
        outside=no_overlap,
        extra=source.stats()
    )

    return tag, no_null_counts, stats


@define(slots=True)
class JoblibMultiReadsCounter(MultiReadsCounter[_T, _K]):
    """
    A class representing a reads counter that uses joblib to parallelize the counting process.
    """
    _sources: dict[_K, Source] = field(alias="sources")
    resolution: Resolution[list[Overlap[_T]], Counts[_T | None]]
    parallel: Parallel
    _counts: dict[_K, Counts[_T]] = field(factory=dict, init=False)
    _stats: list[CountingStats] = field(factory=list, init=False)

    def count(self, data: Iterable[_T], intervals: Iterable[Interval]):
        """
        Count the reads in the given intervals.

        :param data: The data to count.
        :param intervals: Intervals associated with the data.
        """
        _data, _intervals = list(data), list(intervals)
        if len(_data) != len(_intervals):
            raise ValueError(
                f"Data and intervals must have the same length, got {len(_data)} and {len(_intervals)} respectively."
            )

        # Group data into local partitions to support the parallel counting
        partitions = Partition.from_intervals(_data, _intervals)

        # Run the counting process
        results = self.parallel(
            delayed(run)(tag, copy.deepcopy(source), partition, self.resolution)
            for partition in partitions
            for tag, source in self._sources.items()
        )

        # Merge counts & stats
        for tag, counts, stats in results:
            _counts = self._counts.setdefault(tag, {})
            for k, v in counts.items():
                _counts[k] = _counts.get(k, 0) + v

            self._stats.append(stats)

    def counts(self) -> dict[_K, Counts[_T]]:
        return self._counts

    # TODO: I want counts as dict kind of things!!!

    def sources(self) -> dict[_K, Source]:
        return self._sources

    def stats(self) -> Iterable[CountingStats]:
        return self._stats

    def reset(self):
        self._counts.clear()
        self._stats.clear()
