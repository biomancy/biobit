from typing import Self

from biobit.core.loc import IntoInterval, IntoOrientation
from biobit.core.ngs import Layout
from biobit.io.bam import IntoReader
from biobit.toolkit.countit.result import Counts
from .resolution import IntoResolution


class EngineBuilder[E]:
    def __init__(self) -> None: ...

    def set_threads(self, threads: int) -> Self: ...

    def add_elements(
            self, elements: list[tuple[E, list[tuple[str, IntoOrientation, list[IntoInterval]]]]]
    ) -> Self: ...

    def add_partitions(self, partitions: list[tuple[str, IntoInterval]]) -> Self: ...

    def build(self) -> Engine[E]: ...


class Engine[E]:
    @staticmethod
    def builder() -> EngineBuilder[E]: ...

    def run[S](self, sources: list[tuple[S, IntoReader, Layout]], resolution: IntoResolution) -> list[Counts[S, E]]: ...
