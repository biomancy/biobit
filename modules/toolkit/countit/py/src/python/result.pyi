from types import GenericAlias
from typing import final

from biobit.core.loc import Interval

@final
class ResolutionOutcome:
    resolved: float
    discarded: float

@final
class PartitionMetrics:
    contig: str
    interval: Interval
    time_s: float
    outcomes: ResolutionOutcome

    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

@final
class Counts[S, E]:
    @classmethod
    def __class_getitem__(cls, args: object) -> GenericAlias: ...

    source: S
    elements: list[E]
    counts: list[float]
    partitions: list[PartitionMetrics]

    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

__all__ = ["Counts", "PartitionMetrics", "ResolutionOutcome"]
