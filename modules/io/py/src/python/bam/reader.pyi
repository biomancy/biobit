from pathlib import Path
from typing import Self, TypeAlias, final

@final
class Reader:
    filename: str
    inflags: int
    exflags: int
    minmapq: int
    batch_size: int

    def __new__(
        cls,
        filename: str | Path,
        inflags: int = 0,
        exflags: int = 516,
        minmapq: int = 0,
        batch_size: int = 1024,
    ) -> Self: ...
    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

IntoReader: TypeAlias = Reader | str

__all__ = ["IntoReader", "Reader"]
