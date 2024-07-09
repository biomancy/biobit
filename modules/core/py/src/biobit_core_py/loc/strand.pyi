from typing import Literal, TypeVar, Generic

from .orientation import Orientation

StrandLike = Strand | Literal["+", "-", 1, -1]


class Strand:
    Forward = 1
    Reverse = -1

    def __init__(self, value: int | str | Strand) -> None: ...

    def flip(self) -> None: ...

    def flipped(self) -> Strand: ...

    def symbol(self) -> str: ...

    def to_orientation(self) -> Orientation: ...

    def __repr__(self) -> str: ...

    def __str__(self) -> str: ...

    def __hash__(self) -> int: ...


_T = TypeVar("_T")


class Stranded(Generic[_T]):
    fwd: _T
    rev: _T
