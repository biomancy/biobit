from typing import Any

from biobit.core.ngs import Layout
from biobit.io.bam import IntoReader
from .result import Harvest
from .workload import Workload


class Reaper:
    def __init__(self, threads: int = -1) -> None: ...

    def add_source(self, sample: Any, source: IntoReader, layout: Layout) -> Reaper: ...

    def add_sources(self, sample: Any, sources: list[IntoReader], layout: Layout) -> Reaper: ...

    def add_comparison(self, tag: Any, signal: Any, control: Any, workload: Workload) -> Reaper: ...

    def run(self) -> list[Harvest]: ...

    def reset(self) -> Reaper: ...
