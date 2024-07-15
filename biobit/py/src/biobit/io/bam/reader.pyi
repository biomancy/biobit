from pathlib import Path


class Reader:
    filename: str
    inflags: int
    exflags: int
    minmapq: int
    batch_size: int

    def __init__(self, filename: str | Path, inflags: int = 0, exflags: int = 516, minmapq: int = 0,
                 batch_size: int = 1024) -> None: ...

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore


IntoReader = Reader | str | Path
