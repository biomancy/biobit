from typing import TypeVar, Generic

from .strand import IntoStrand

T = TypeVar('T')


class PerStrand(Generic[T]):
    forward: T
    reverse: T

    def __init__(self, /, forward: T, reverse: T): ...

    def __getitem__(self, strand: IntoStrand) -> T: ...

    def __setitem__(self, key: IntoStrand, value: T) -> None: ...
