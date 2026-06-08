from collections.abc import Sequence

from biobit.core.loc import Orientation

from .pileup import SparsePileup

class SelectedPileup[T]:
    """
    REAT output for one sample tag.

    Pileups are grouped by exact `(seqid, orientation)` keys. Missing keys mean no sites were
    selected for that sequence/orientation. `pileups()` builds and returns a new dictionary each
    time; mutating it does not mutate the underlying result object.
    """

    tag: T

    def __init__(self, tag: T, pileups: Sequence[SparsePileup]) -> None:
        """Create a selected-pileup result from sparse pileup chunks."""
        ...

    def pileups(self) -> dict[tuple[str, Orientation], SparsePileup]:
        """Return pileups grouped by `(seqid, orientation)`."""
        ...

    def len(self) -> int: ...
    def is_empty(self) -> bool: ...

    __hash__ = None  # type: ignore
