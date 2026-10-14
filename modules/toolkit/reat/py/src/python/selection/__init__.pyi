from collections.abc import Sequence
from typing import Self, final

from biobit.core.loc import IntoInterval, IntoOrientation

@final
class Mismatches:
    """
    Select sites whose non-reference observations pass mismatch thresholds.

    A site is selected when mismatch count is at least
    `minmismatches`, and mismatch frequency is at least `minfreq`.
    """

    minmismatches: int
    minfreq: float

    def __new__(cls, minmismatches: int = 1, minfreq: float = 0.0) -> Self:
        """Create a mismatch selector. `minfreq` must be in `[0.0, 1.0]`."""

    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

@final
class RequiredSites:
    """
    Select explicitly required genomic sites.

    Required intervals are keyed by both sequence ID and orientation. Selection is orientation
    strict: a `("+")` required interval does not select reverse or dual pileups, and `("=")`
    only matches dual pileups. Intervals with the same `(seqid, orientation)` are sorted and
    merged, including touching intervals.
    """

    len: int
    """Number of merged required intervals across all sequence/orientation keys."""

    def __new__(
        cls,
        required: Sequence[tuple[str, IntoOrientation, Sequence[IntoInterval]]]
        | None = None,
    ) -> Self:
        """
        Build a required-site selector.

        Empty interval lists are ignored. Coordinates are converted to unsigned 64-bit integers.
        """

    def is_empty(self) -> bool: ...
    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

@final
class RequiredOrMismatches:
    """
    Union selector: required sites are selected, and mismatch-passing sites are selected.

    The two inner selectors are applied independently, so a site matching either criterion is kept.
    """

    def __new__(
        cls,
        required: RequiredSites | None = None,
        mismatches: Mismatches | None = None,
    ) -> Self: ...
    def required(self) -> RequiredSites: ...
    def mismatches(self) -> Mismatches: ...
    def __eq__(self, other: object, /) -> bool: ...

    __hash__: None = None  # type: ignore[assignment]

__all__ = ["Mismatches", "RequiredOrMismatches", "RequiredSites"]
