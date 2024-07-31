from typing import Any

from biobit.core.ngs import Layout
from biobit.io.bam import IntoReader
from .config import Config
from .result import Ripped


class Ripper:
    def __init__(self, threads: int = -1) -> None: ...

    def add_partition(self, contig: str, start: int, end: int) -> Ripper: ...

    def add_source(self, sample: Any, source: IntoReader, layout: Layout) -> Ripper: ...

    def add_sources(self, sample: Any, sources: list[IntoReader], layout: Layout) -> Ripper: ...

    def add_comparison(self, tag: Any, signal: Any, control: Any, config: Config) -> Ripper: ...

    def run(self) -> list[Ripped]: ...