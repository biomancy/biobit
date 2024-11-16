from typing import TypeVar, Generic

from intervaltree import IntervalTree

from biobit.core.loc import Interval, Orientation, IntoOrientation
from .overlap import Overlap

_T = TypeVar("_T")


class Bundle(Generic[_T]):
    itrees: dict[tuple[str, Orientation], IntervalTree]

    def __init__(self, itrees: dict[tuple[str, Orientation], IntervalTree] | None = None):
        self.itrees = itrees if itrees else {}

    def set(self, contig: str, orientation: IntoOrientation, index: IntervalTree):
        self.itrees[(contig, Orientation(orientation))] = index

    def overlap(
            self,
            contig: str,
            orientation: IntoOrientation,
            start: int | None = None,
            end: int | None = None,
            rng: Interval | None = None,
    ) -> Overlap[_T]:
        orient = Orientation(orientation)

        if (start is None or end is None) and rng is None:
            raise ValueError("Either start and end or Interval must be provided")
        elif (start is not None or end is not None) and rng is not None:
            raise ValueError("Either start and end or Interval must be provided, not both")

        if rng is None:
            if start is None or end is None:
                raise ValueError("Both start and end must be provided")
            rng = Interval(start, end)

        index = self.itrees.get((contig, orient), None)
        if index is None:
            return Overlap(rng, [], [])

        hits, annotation = [], []
        for it in sorted(index.overlap(rng.start, rng.end), key=lambda x: x.begin):
            intersection = rng.intersection((it.begin, it.end))
            assert intersection is not None
            hits.append(intersection)
            annotation.append(it.data)
        return Overlap(rng, hits, annotation)
