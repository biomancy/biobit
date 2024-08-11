from collections.abc import Iterable
from dataclasses import dataclass
from typing import Generic, TypeVar

from sortedcontainers import SortedList

from biobit.core.loc import Segment

_T = TypeVar("_T")


@dataclass(frozen=True, slots=True)
class OverlapSteps(Generic[_T]):
    rng: Segment
    boundaries: list[Segment]
    annotations: list[set[_T]]

    def __post_init__(self):
        if len(self.boundaries) != len(self.annotations):
            raise ValueError("Boundaries and annotations must have the same length")
        if len(self.boundaries) == 0:
            raise ValueError("Boundaries and annotations must have at least one element")
        if self.boundaries[0].start != self.rng.start or self.boundaries[-1].end != self.rng.end:
            raise ValueError("Boundaries must start at the beginning of the range and end at the end of the range")

    def __len__(self) -> int:
        return len(self.annotations)

    def __iter__(self) -> Iterable[tuple[Segment, set[_T]]]:
        return zip(self.boundaries, self.annotations)


@dataclass(frozen=True, slots=True)
class Overlap(Generic[_T]):
    rng: Segment
    intervals: list[Segment]
    annotations: list[_T]

    def to_steps(self) -> OverlapSteps[_T]:
        template = {self.rng.start, self.rng.end}
        for it in self.intervals:
            template.add(it.start)
            template.add(it.end)

        boundaries = SortedList(template)

        annotation: list[set[_T]] = [set() for _ in range(len(boundaries) - 1)]

        for it, anno in zip(self.intervals, self.annotations):
            st, en = boundaries.bisect_left(it.start), boundaries.bisect_left(it.end)
            for stanno in annotation[st:en]:
                stanno.add(anno)
        steps = [Segment(boundaries[i], boundaries[i + 1]) for i in range(len(boundaries) - 1)]

        return OverlapSteps(self.rng, steps, annotation)

    def __iter__(self) -> Iterable[tuple[Segment, _T]]:
        return zip(self.intervals, self.annotations)

    def __len__(self) -> int:
        return len(self.intervals)
