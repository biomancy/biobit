from typing import Any

from biobit.core.loc import Segment, PerOrientation


class Peak:
    @property
    def start(self) -> int: ...

    @property
    def end(self) -> int: ...

    @property
    def value(self) -> float: ...

    @property
    def summit(self) -> int: ...


class Region:
    @property
    def contig(self) -> str: ...

    @property
    def segment(self) -> Segment: ...

    @property
    def peaks(self) -> PerOrientation[list[Peak]]: ...


class Ripped:
    @property
    def tag(self) -> Any: ...

    @property
    def regions(self) -> list[Region]: ...
