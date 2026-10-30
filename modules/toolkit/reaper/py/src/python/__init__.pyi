from typing import Any, Self, final

from biobit.core.loc import Interval, Orientation
from biobit.io.bam import IntoReader
from biobit.toolkit.bioproj import Layout

from . import cmp as cmp
from . import model as model
from . import pcalling as pcalling
from . import postfilter as postfilter

@final
class Reaper:
    def __new__(cls, threads: int = -1) -> Self: ...
    def add_source(self, tag: Any, source: IntoReader, layout: Layout) -> Reaper: ...
    def add_sources(
        self, sample: Any, sources: list[IntoReader], layout: Layout
    ) -> Reaper: ...
    def add_comparison(
        self, tag: Any, signal: Any, control: Any, workload: Workload
    ) -> Reaper: ...
    def run(self) -> list[Harvest]: ...
    def reset(self) -> Reaper: ...

@final
class Config:
    def __new__(
        cls,
        model: model.RNAPileup,
        cmp: cmp.Enrichment,
        pcalling: pcalling.ByCutoff,
        postfilter: postfilter.NMS,
    ) -> Self: ...

@final
class Workload:
    def __new__(cls) -> Self: ...
    def add_region(
        self, contig: str, start: int, end: int, config: Config
    ) -> Workload: ...
    def add_regions(
        self, regions: list[tuple[str, int, int]], config: Config
    ) -> Workload: ...

@final
class Peak:
    @property
    def interval(self) -> Interval: ...
    @property
    def value(self) -> float: ...
    @property
    def summit(self) -> int: ...

@final
class HarvestRegion:
    @property
    def contig(self) -> str: ...
    @property
    def orientation(self) -> Orientation: ...
    @property
    def interval(self) -> Interval: ...
    @property
    def signal(self) -> list[Interval]: ...
    @property
    def control(self) -> list[Interval]: ...
    @property
    def modeled(self) -> list[Interval]: ...
    @property
    def raw_peaks(self) -> list[Peak]: ...
    @property
    def filtered_peaks(self) -> list[Peak]: ...

@final
class Harvest:
    @property
    def comparison(self) -> Any: ...
    @property
    def regions(self) -> list[HarvestRegion]: ...

__all__ = [
    "Config",
    "Harvest",
    "HarvestRegion",
    "Peak",
    "Reaper",
    "Workload",
    "cmp",
    "model",
    "pcalling",
    "postfilter",
]
