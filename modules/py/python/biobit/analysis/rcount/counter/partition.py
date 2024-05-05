from collections import defaultdict
from typing import TypeVar, Iterable, Generic

from attrs import define

from biobit import algo
from biobit.core import Interval

_T = TypeVar('_T')


@define(slots=True, frozen=True)
class Partition(Generic[_T]):
    """
    A class representing a partition of intervals.

    Attributes:
        contig: The contig of the partition.
        intervals: The intervals in the partition.
        data: The data corresponding to the intervals.
    """
    contig: str
    intervals: tuple[Interval, ...]
    data: tuple[_T, ...]

    def __attrs_post_init__(self):
        if len(self.intervals) != len(self.data):
            raise ValueError("Intervals and data lists must have the same length")

    @staticmethod
    def from_intervals(data: Iterable[_T], intervals: Iterable[Interval], maxdist: int = 1024) -> list["Partition"]:
        """
        Partition intervals into groups based on contig, strand, and their distance from each other.

        :param data: An iterable of data objects.
        :param intervals: An iterable of intervals associated with each data object.
        :param maxdist: The maximum distance between intervals for them to be considered part of the same partition.
        """
        # Group by contig
        partition = defaultdict(list)
        for d, i in zip(data, intervals):
            partition[i.contig].append((i, d))

        # Sort by start position and make final partitions
        results = []
        for contig, part in partition.items():
            part.sort(key=lambda x: x[0].rng.start)
            partitions = algo.misc.group_within(
                part,
                distance=lambda x, y: min(
                    abs(y[0].rng.start - x[0].rng.start),
                    abs(y[0].rng.end - x[0].rng.start),
                    abs(y[0].rng.start - x[0].rng.end),
                    abs(y[0].rng.end - x[0].rng.end)
                ),
                maxdist=maxdist
            )

            for p in partitions:
                pinterval, pdata = zip(*p)
                results.append(Partition(contig, pinterval, pdata))
        return results

    def __len__(self) -> int:
        return len(self.intervals)
