from types import GenericAlias
from typing import Self, final

from biobit.core.loc import IntoInterval, IntoOrientation
from biobit.io.bam import IntoReader
from biobit.toolkit.bioproj import Layout
from biobit.toolkit.countit.result import Counts

from .resolution import IntoResolution

@final
class EngineBuilder[E]:
    @classmethod
    def __class_getitem__(cls, args: object) -> GenericAlias: ...
    def __new__(cls) -> Self: ...
    def set_threads(self, threads: int) -> Self: ...
    def add_elements(
        self,
        elements: list[tuple[E, list[tuple[str, IntoOrientation, list[IntoInterval]]]]],
    ) -> Self: ...
    def add_partitions(self, partitions: list[tuple[str, IntoInterval]]) -> Self: ...
    def build(self) -> Engine[E]: ...

@final
class Engine[E]:
    @classmethod
    def __class_getitem__(cls, args: object) -> GenericAlias: ...
    @staticmethod
    def builder() -> EngineBuilder[E]: ...
    def run[S](
        self, sources: list[tuple[S, IntoReader, Layout]], resolution: IntoResolution
    ) -> list[Counts[S, E]]: ...

__all__ = ["Engine", "EngineBuilder"]
