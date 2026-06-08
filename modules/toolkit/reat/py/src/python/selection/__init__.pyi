from collections.abc import Sequence

from biobit.core.loc import IntoInterval, IntoOrientation

class Mismatches:
    """
    Select sites whose non-reference observations pass mismatch thresholds.

    A site is selected when coverage is at least `mincov`, mismatch count is at least
    `minmismatches`, and mismatch frequency is at least `minfreq`.
    """

    minmismatches: int
    minfreq: float
    mincov: int

    def __init__(
        self, minmismatches: int = 1, minfreq: float = 0.0, mincov: int = 1
    ) -> None:
        """Create a mismatch selector. `minfreq` must be in `[0.0, 1.0]`."""
        ...

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

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

    def __init__(
        self,
        required: Sequence[tuple[str, IntoOrientation, Sequence[IntoInterval]]]
        | None = None,
    ) -> None:
        """
        Build a required-site selector.

        Empty interval lists are ignored. Coordinates are converted to unsigned 64-bit integers.
        """
        ...

    def is_empty(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class RequiredOrMismatches:
    """
    Union selector: required sites are selected, and mismatch-passing sites are selected.

    The two inner selectors are applied independently, so a site matching either criterion is kept.
    """

    required: RequiredSites
    mismatches: Mismatches

    def __init__(
        self,
        required: RequiredSites | None = None,
        mismatches: Mismatches | None = None,
    ) -> None: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
