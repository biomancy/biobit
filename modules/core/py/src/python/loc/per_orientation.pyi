from types import GenericAlias
from typing import Self, final

from .orientation import IntoOrientation

@final
class PerOrientation[T]:
    @classmethod
    def __class_getitem__(cls, args: object) -> GenericAlias: ...

    forward: T
    reverse: T
    dual: T

    def __new__(cls, /, forward: T, reverse: T, dual: T) -> Self: ...
    def get(self, orientation: IntoOrientation) -> T: ...
    def __getitem__(self, orientation: IntoOrientation, /) -> T: ...
    def __setitem__(self, orientation: IntoOrientation, value: T, /) -> None: ...

__all__ = ["PerOrientation"]
