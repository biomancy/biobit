from typing import Any

from biobit.core.loc import Segment, Orientation


class Peak:
    @property
    def segment(self) -> Segment: ...

    @property
    def value(self) -> float: ...

    @property
    def summit(self) -> int: ...


class HarvestRegion:
    @property
    def contig(self) -> str: ...

    @property
    def orientation(self) -> Orientation: ...

    @property
    def segment(self) -> Segment: ...

    @property
    def signal(self) -> list[Segment]: ...

    @property
    def control(self) -> list[Segment]: ...

    @property
    def modeled(self) -> list[Segment]: ...

    @property
    def raw_peaks(self) -> list[Peak]: ...

    @property
    def filtered_peaks(self) -> list[Peak]: ...


class Harvest:
    @property
    def comparison(self) -> Any: ...

    @property
    def regions(self) -> list[HarvestRegion]: ...
