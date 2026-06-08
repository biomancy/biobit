from biobit.core.loc import Interval, IntoOrientation, Orientation

class Pileup:
    a: list[int]
    c: list[int]
    g: list[int]
    t: list[int]
    n: list[int]
    deletion: list[int]
    coverage: list[int]

    def __init__(
        self,
        a: list[int],
        c: list[int],
        g: list[int],
        t: list[int],
        n: list[int],
        deletion: list[int],
    ) -> None: ...
    @staticmethod
    def zeros(len: int) -> Pileup: ...
    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class SparsePileup:
    seqid: str
    orientation: Orientation
    positions: list[int]
    counts: Pileup
    interval: Interval

    def __init__(
        self,
        seqid: str,
        orientation: IntoOrientation,
        positions: list[int],
        counts: Pileup,
    ) -> None: ...
    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
