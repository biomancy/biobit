from .cmp import Enrichment
from .model import RNAPileup
from .pcalling import ByCutoff
from .postfilter import NMS


class Config:
    def __init__(self, model: RNAPileup, cmp: Enrichment, pcalling: ByCutoff, postfilter: NMS) -> None: ...


class Workload:
    def __init__(self) -> None: ...

    def add_region(self, contig: str, start: int, end: int, config: Config) -> Workload: ...

    def add_regions(self, regions: list[tuple[str, int, int]], config: Config) -> Workload: ...
