from typing import TypeVar, Generic

from .orientation import IntoOrientation

T = TypeVar('T')


class PerOrientation(Generic[T]):
    forward: T
    reverse: T
    dual: T

    def __init__(self, /, forward: T, reverse: T, dual: T): ...

    def __getitem__(self, orientation: IntoOrientation) -> T: ...

    def __setitem__(self, key: IntoOrientation, value: T) -> None: ...
