from pathlib import Path
from typing import Self, final

@final
class Reader:
    def __new__(
        cls,
        filename: str | Path,
        inflags: int = 0,
        exflags: int = 516,
        minmapq: int = 0,
        batch_size: int = 1024,
    ) -> Self: ...
    @property
    def filename(self) -> Path: ...
    @property
    def inflags(self) -> int: ...
    @property
    def exflags(self) -> int: ...
    @property
    def minmapq(self) -> int: ...
    @property
    def batch_size(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

type IntoReader = Reader | str

__all__ = ["IntoReader", "Reader"]
