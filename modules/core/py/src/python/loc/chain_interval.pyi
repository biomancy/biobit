from collections.abc import Iterator, Sequence
from typing import Self, TypeAlias, final

from .interval import Interval, IntoInterval

@final
class ChainInterval:
    def __new__(cls, source: IntoChainInterval) -> Self: ...
    def __iter__(self) -> Iterator[Interval]: ...

IntoChainInterval: TypeAlias = ChainInterval | Sequence[IntoInterval]

__all__ = ["ChainInterval", "IntoChainInterval"]
