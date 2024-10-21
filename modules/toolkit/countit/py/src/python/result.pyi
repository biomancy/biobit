from typing import Any

from biobit.core.loc import Segment


class ResolutionOutcome:
    resolved: float
    discarded: float


class PartitionMetrics:
    contig: str
    segment: Segment
    time_s: float
    outcomes: ResolutionOutcome

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore


class Counts[S, E]:
    source: S
    elements: list[E]
    counts: list[float]
    partitions: list[PartitionMetrics]

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
