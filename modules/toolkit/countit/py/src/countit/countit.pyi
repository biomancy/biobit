from typing import Any

from biobit.core import ngs
from biobit.core.loc import IntoOrientation, IntoSegment, IntoLocus
from biobit.core.loc import Segment
from biobit.io import bam


class CountIt:
    def __init__(self, threads: int = -1) -> None: ...

    def add_annotation(self, data: Any, intervals: list[tuple[str, IntoOrientation, list[IntoSegment]]]) -> CountIt: ...

    def add_partition(self, partition: IntoLocus) -> CountIt: ...

    def add_source(self, tag: Any, source: bam.Reader, layout: ngs.Layout) -> CountIt: ...

    def run(self) -> list[Counts]: ...

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore


class Stats:
    contig: str
    segment: Segment
    time_s: float
    inside_annotation: float
    outside_annotation: float

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore


class Counts:
    source: Any
    data: list[Any]
    counts: list[float]
    stats: list[Stats]

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
