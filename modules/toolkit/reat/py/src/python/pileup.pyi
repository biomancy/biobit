from collections.abc import Sequence

from biobit.core.loc import Interval, IntoOrientation, Orientation

class Pileup:
    """
    Per-position nucleotide/deletion count arrays.

    Count arrays use unsigned 32-bit integers (internally). All six arrays must have identical length.
    Property accessors return copies as Python lists. Therefore, caching access is highly recommended.

    Note that mutating the returned lists does not affect the pileup, because only copies are returned.
    """

    a: list[int]
    c: list[int]
    g: list[int]
    t: list[int]
    n: list[int]
    deletion: list[int]
    coverage: list[int]

    def __init__(
        self,
        a: Sequence[int],
        c: Sequence[int],
        g: Sequence[int],
        t: Sequence[int],
        n: Sequence[int],
        deletion: Sequence[int],
    ) -> None:
        """Create a pileup from equal-length count arrays."""
        ...

    @staticmethod
    def zeros(len: int) -> Pileup:
        """Create a zero-filled pileup of `len` sites."""
        ...

    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class SparsePileup:
    """
    Sparse pileup over selected genomic positions for one `(seqid, orientation)`.

    `positions` must be sorted, unique, non-empty unsigned 64-bit coordinates. `counts.len()` must
    match `len(positions)`. The `interval` property spans from the first selected position to one
    past the last selected position. I.e., `interval.start == positions[0]` and `interval.end == positions[-1] + 1`.
    """

    seqid: str
    orientation: Orientation
    positions: list[int]
    counts: Pileup
    interval: Interval

    def __init__(
        self,
        seqid: str,
        orientation: IntoOrientation,
        positions: Sequence[int],
        counts: Pileup,
    ) -> None:
        """Create a sparse pileup from positions and matching count arrays."""
        ...

    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
