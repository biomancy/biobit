from typing import Self

from biobit.core.ngs import Layout
from biobit.io.bam import IntoReader
from biobit.io.fasta import IndexedSources

from .result import SelectedPileup
from .selection import Mismatches, RequiredOrMismatches, RequiredSites
from .task import Task

Selector = Mismatches | RequiredSites | RequiredOrMismatches

class Reat[T]:
    def __init__(
        self,
        reference: IndexedSources,
        selector: Selector | None = None,
        min_phred: int = 20,
        threads: int = -1,
    ) -> None: ...
    def add_sources(
        self, tag: T, sources: list[IntoReader], layout: Layout
    ) -> Self: ...
    def run(self, tasks: list[Task]) -> list[SelectedPileup[T]]: ...
    def reset(self) -> Self: ...

    __hash__ = None  # type: ignore
