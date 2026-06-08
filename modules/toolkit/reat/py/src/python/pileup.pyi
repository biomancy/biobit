from collections.abc import Sequence

from biobit.core.loc import Interval

class Pileup:
    """
    Per-position nucleotide/deletion count arrays.

    Count arrays use unsigned 32-bit integers (internally). All six arrays must have identical length.
    Count accessors return copies as Python lists. Therefore, caching access is highly recommended.

    Note that mutating the returned lists does not affect the pileup, because only copies are returned.
    """

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
    def a(self) -> list[int]: ...
    def c(self) -> list[int]: ...
    def g(self) -> list[int]: ...
    def t(self) -> list[int]: ...
    def n(self) -> list[int]: ...
    def deletion(self) -> list[int]: ...
    def coverage(self) -> list[int]: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class SparsePileup:
    """
    Sparse pileup over selected genomic positions.

    `positions` must be sorted, unique, non-empty unsigned 64-bit coordinates. `counts.len()` must
    match `len(positions)`. The `interval` property spans from the first selected position to one
    past the last selected position. I.e., `interval.start == positions[0]` and `interval.end == positions[-1] + 1`.
    """

    interval: Interval

    def __init__(
        self,
        positions: Sequence[int],
        counts: Pileup,
    ) -> None:
        """Create a sparse pileup from positions and matching count arrays."""
        ...

    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def positions(self) -> list[int]: ...
    def counts(self) -> Pileup: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class TaskPileup:
    """
    Sparse pileup plus reference bases for the same selected positions.

    `reference` is aligned one-to-one with `pileup.positions()`.
    """

    interval: Interval

    def __init__(self, pileup: SparsePileup, reference: Sequence[str]) -> None:
        """Create a task pileup from a sparse pileup and matching reference bases."""
        ...

    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def pileup(self) -> SparsePileup: ...
    def reference(self) -> list[str]: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
