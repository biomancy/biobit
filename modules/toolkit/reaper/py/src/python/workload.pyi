from typing import Self, final

from .cmp import Enrichment
from .model import RNAPileup
from .pcalling import ByCutoff
from .postfilter import NMS

@final
class Config:
    def __new__(
        cls, model: RNAPileup, cmp: Enrichment, pcalling: ByCutoff, postfilter: NMS
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

__all__ = ["Config", "Workload"]
