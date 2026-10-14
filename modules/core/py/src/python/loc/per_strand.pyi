from types import GenericAlias
from typing import Self, final

from .strand import IntoStrand

@final
class PerStrand[T]:
    @classmethod
    def __class_getitem__(cls, args: object) -> GenericAlias: ...

    forward: T
    reverse: T

    def __new__(cls, /, forward: T, reverse: T) -> Self: ...
    def get(self, strand: IntoStrand) -> T: ...
    def __getitem__(self, strand: IntoStrand, /) -> T: ...
    def __setitem__(self, strand: IntoStrand, value: T, /) -> None: ...

__all__ = ["PerStrand"]
