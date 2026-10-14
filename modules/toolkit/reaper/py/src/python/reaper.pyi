from typing import Any, Self, final

from biobit.io.bam import IntoReader
from biobit.toolkit.bioproj import Layout

from .result import Harvest
from .workload import Workload

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

__all__ = ["Reaper"]
