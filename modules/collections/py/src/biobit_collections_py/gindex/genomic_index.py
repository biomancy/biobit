from collections import defaultdict
from collections.abc import Iterable
from pathlib import Path
from typing import Callable, TypeVar, Generic

from intervaltree import IntervalTree
from pybedtools import BedTool, Interval as BedInterval

from biobit.core import Range, Orientation, OrientationLike, Interval
from .overlap import Overlap

_T = TypeVar("_T")


def bedname(it: BedInterval) -> str:
    return it.name


def merge(*indices: "GenomicIndex[_T]") -> "GenomicIndex[_T]":
    if not indices:
        raise ValueError("Can't merge empty list of genomic indexes")
    elif len(indices) == 1:
        return indices[0]

    keys: set[tuple[str, Orientation]] = set()
    for ind in indices:
        for key in ind.itrees.keys():
            keys.add(key)

    itrees = {}
    for key in keys:
        merged = set()
        for ind in indices:
            tree = ind.itrees.get(key, None)
            if tree is not None:
                merged |= set(tree)

        itrees[key] = IntervalTree(merged)
    return GenomicIndex(itrees)


class GenomicIndex(Generic[_T]):
    itrees: dict[tuple[str, Orientation], IntervalTree]

    def __init__(self, itrees: dict[tuple[str, Orientation], IntervalTree] | None = None):
        self.itrees = itrees if itrees else {}

    def set(self, contig: str, orientation: OrientationLike, index: IntervalTree):
        self.itrees[(contig, Orientation.normalize(orientation))] = index

    def overlap(
            self,
            contig: str,
            orientation: OrientationLike,
            start: int | None = None,
            end: int | None = None,
            rng: Range | None = None,
    ) -> Overlap[_T]:
        orient = Orientation.normalize(orientation)

        if (start is None or end is None) and rng is None:
            raise ValueError("Either start and end or range must be provided")
        elif (start is not None or end is not None) and rng is not None:
            raise ValueError("Either start and end or range must be provided, not both")

        if rng is None:
            if start is None or end is None:
                raise ValueError("Both start and end must be provided")
            rng = Range(start, end)

        index = self.itrees.get((contig, orient), None)
        if index is None:
            return Overlap(rng, [], [])

        hits, annotation = [], []
        for it in sorted(index.overlap(rng.start, rng.end), key=lambda x: x.begin):
            ov = rng.overlap(start=it.begin, end=it.end)
            assert ov is not None
            hits.append(ov.rng)
            annotation.append(it.data)
        return Overlap(rng, hits, annotation)

    def merge(self, *indices: 'GenomicIndex[_T]', inplace: bool = False) -> 'GenomicIndex[_T]':
        merged = merge(self, *indices)
        if inplace:
            self.itrees = merged.itrees
            return self
        else:
            return merged

    def subset(
            self, contig: str, start: int, end: int, orientation: OrientationLike | None = None
    ) -> "GenomicIndex[_T]":
        if orientation is None:
            orients = [Orientation.fwd, Orientation.rev, Orientation.dual]
        else:
            orients = [Orientation.normalize(orientation)]

        trees = {}
        for orient in orients:
            index = self.itrees.get((contig, orient), None)
            if index is not None:
                tree = IntervalTree()
                for it in index.overlap(start, end):
                    tree.addi(it.begin, it.end, data=it.data)
                trees[(contig, orient)] = tree

        return GenomicIndex(trees)

    def covered(self) -> Iterable[Interval]:
        for (contig, orientation), tree in self.itrees.items():
            for it in tree.all_intervals:
                yield Interval(contig, Range(it.begin, it.end), orientation)

    @staticmethod
    def from_bed(
            bed: Path,
            datafn: Callable[[BedInterval], _T] = bedname,  # type: ignore
            filterfn: Callable[[BedInterval], bool] | None = None,
    ) -> "GenomicIndex[_T]":
        filterfn = filterfn if filterfn is not None else (lambda _: True)

        itrees: defaultdict[tuple[str, Orientation], IntervalTree] = defaultdict(IntervalTree)
        for it in BedTool(bed):
            if not filterfn(it):
                continue
            key = (it.chrom, Orientation.normalize(it.strand))
            itrees[key].addi(it.start, it.end, data=datafn(it))
        result: dict[tuple[str, Orientation], IntervalTree] = dict(itrees)
        return GenomicIndex(result)
