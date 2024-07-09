from typing import Literal

from .strand import Strand

OrientationLike = Orientation | Literal["+", "-", "=", 1, -1, 0]


class Orientation:
    Forward = 1
    Reverse = -1
    Dual = 0

    def __init__(self, value: int | str | Orientation) -> None: ...

    def flip(self) -> None: ...

    def flipped(self) -> Orientation: ...

    def symbol(self) -> str: ...

    def to_strand(self) -> Strand: ...

    def __repr__(self) -> str: ...

    def __str__(self) -> str: ...

    def __hash__(self) -> int: ...
